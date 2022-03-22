use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::LibraryModel;
use crate::app::components::utils::wrap_flowbox_item;
use crate::app::components::{AlbumWidget, Component, EventListener};
use crate::app::dispatch::Worker;
use crate::app::models::AlbumModel;
use crate::app::state::LoginEvent;
use crate::app::{AppEvent, BrowserEvent, ListStore};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/library.ui")]
    pub struct LibraryWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub flowbox: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub status_page: TemplateChild<libadwaita::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LibraryWidget {
        const NAME: &'static str = "LibraryWidget";
        type Type = super::LibraryWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LibraryWidget {}
    impl WidgetImpl for LibraryWidget {}
    impl BoxImpl for LibraryWidget {}
}

glib::wrapper! {
    pub struct LibraryWidget(ObjectSubclass<imp::LibraryWidget>) @extends gtk::Widget, gtk::Box;
}

impl LibraryWidget {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of LibraryWidget")
    }

    fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::LibraryWidget::from_instance(self)
            .scrolled_window
            .connect_edge_reached(move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    f()
                }
            });
    }

    fn bind_albums<F>(&self, worker: Worker, store: &ListStore<AlbumModel>, on_album_pressed: F)
    where
        F: Fn(String) + Clone + 'static,
    {
        imp::LibraryWidget::from_instance(self).flowbox.bind_model(
            Some(store.unsafe_store()),
            move |item| {
                wrap_flowbox_item(item, |album_model| {
                    let f = on_album_pressed.clone();
                    let album = AlbumWidget::for_model(album_model, worker.clone());
                    album.connect_album_pressed(clone!(@weak album_model => move |_| {
                        f(album_model.uri());
                    }));
                    album
                })
            },
        );
    }

    pub fn status_page(&self) -> &libadwaita::StatusPage {
        &imp::LibraryWidget::from_instance(self).status_page
    }
}

pub struct Library {
    widget: LibraryWidget,
    worker: Worker,
    model: Rc<LibraryModel>,
}

impl Library {
    pub fn new(worker: Worker, model: LibraryModel) -> Self {
        let model = Rc::new(model);
        let widget = LibraryWidget::new();
        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more_albums();
        }));

        Self {
            widget,
            worker,
            model,
        }
    }

    fn bind_flowbox(&self) {
        self.widget.bind_albums(
            self.worker.clone(),
            &*self.model.get_list_store().unwrap(),
            clone!(@weak self.model as model => move |id| {
                model.open_album(id);
            }),
        );
    }
}

impl EventListener for Library {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                let _ = self.model.refresh_saved_albums();
                self.bind_flowbox();
            }
            AppEvent::LoginEvent(LoginEvent::LoginCompleted(_)) => {
                let _ = self.model.refresh_saved_albums();
            }
            AppEvent::BrowserEvent(BrowserEvent::LibraryUpdated) => {
                self.widget
                    .status_page()
                    .set_visible(!self.model.has_albums());
            }
            _ => {}
        }
    }
}

impl Component for Library {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.as_ref()
    }
}
