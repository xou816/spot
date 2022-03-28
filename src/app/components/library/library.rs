use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::LibraryModel;
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
        pub gridview: TemplateChild<gtk::GridView>,

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

    fn set_model<F>(&self, store: &ListStore<AlbumModel>, worker: Worker, on_clicked: F)
    where
        F: Fn(String) + Clone + 'static
    {
        let gridview = &imp::LibraryWidget::from_instance(self).gridview;

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, list_item| {
            list_item.set_child(Some(&AlbumWidget::new()));
            // Onclick is handled by album and this causes two layers of highlight on hover
            list_item.set_activatable(false);
        });

        factory.connect_bind(move |factory, list_item| {
            AlbumWidget::bind_list_item(factory, list_item, worker.clone(), on_clicked.clone());
        });

        factory.connect_unbind(move |factory, list_item| {
            AlbumWidget::unbind_list_item(factory, list_item);
        });

        gridview.set_factory(Some(&factory));
        gridview.set_model(Some(&gtk::NoSelection::new(Some(store.unsafe_store()))));
    }

    pub fn status_page(&self) -> &libadwaita::StatusPage {
        &imp::LibraryWidget::from_instance(self).status_page
    }
}

pub struct Library {
    widget: LibraryWidget,
    model: Rc<LibraryModel>,
}

impl Library {
    pub fn new(worker: Worker, model: LibraryModel) -> Self {
        let model = Rc::new(model);
        let widget = LibraryWidget::new();

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more_albums();
        }));
        widget.set_model(
            &model.get_list_store().unwrap(),
            worker.clone(),
            clone!(@weak model => move |uri| {
                model.open_album(uri);
            })
        );

        Self {
            widget,
            model,
        }
    }
}

impl EventListener for Library {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                let _ = self.model.refresh_saved_albums();
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
