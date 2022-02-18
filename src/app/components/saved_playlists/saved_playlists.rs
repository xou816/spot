use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::SavedPlaylistsModel;
use crate::app::components::{AlbumWidget, Component, EventListener};
use crate::app::dispatch::Worker;
use crate::app::models::AlbumModel;
use crate::app::state::LoginEvent;
use crate::app::{AppEvent, BrowserEvent, ListStore};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/saved_playlists.ui")]
    pub struct SavedPlaylistsWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub gridview: TemplateChild<gtk::GridView>,

        #[template_child]
        pub status_page: TemplateChild<libadwaita::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SavedPlaylistsWidget {
        const NAME: &'static str = "SavedPlaylistsWidget";
        type Type = super::SavedPlaylistsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SavedPlaylistsWidget {}
    impl WidgetImpl for SavedPlaylistsWidget {}
    impl BoxImpl for SavedPlaylistsWidget {}
}

glib::wrapper! {
    pub struct SavedPlaylistsWidget(ObjectSubclass<imp::SavedPlaylistsWidget>) @extends gtk::Widget, gtk::Box;
}

impl SavedPlaylistsWidget {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of SavedPlaylistsWidget")
    }

    fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::SavedPlaylistsWidget::from_instance(self)
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
        let gridview = &imp::SavedPlaylistsWidget::from_instance(self).gridview;

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

    pub fn get_status_page(&self) -> &libadwaita::StatusPage {
        &imp::SavedPlaylistsWidget::from_instance(self).status_page
    }
}

pub struct SavedPlaylists {
    widget: SavedPlaylistsWidget,
    model: Rc<SavedPlaylistsModel>,
}

impl SavedPlaylists {
    pub fn new(worker: Worker, model: SavedPlaylistsModel) -> Self {
        let model = Rc::new(model);
        let widget = SavedPlaylistsWidget::new();

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more_playlists();
        }));
        widget.set_model(
            &model.get_list_store().unwrap(),
            worker.clone(),
            clone!(@weak model => move |uri| {
                model.open_playlist(uri);
            })
        );

        Self {
            widget,
            model,
        }
    }
}

impl EventListener for SavedPlaylists {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                let _ = self.model.refresh_saved_playlists();
            }
            AppEvent::LoginEvent(LoginEvent::LoginCompleted(_)) => {
                let _ = self.model.refresh_saved_playlists();
            }
            AppEvent::BrowserEvent(BrowserEvent::SavedPlaylistsUpdated) => {
                self.widget
                    .get_status_page()
                    .set_visible(!self.model.has_playlists());
            }
            _ => {}
        }
    }
}

impl Component for SavedPlaylists {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.as_ref()
    }
}
