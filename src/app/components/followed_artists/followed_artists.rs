use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::FollowedArtistsModel;
use crate::app::components::artist::ArtistWidget;
use crate::app::components::{Component, EventListener};
use crate::app::dispatch::Worker;
use crate::app::models::ArtistModel;
use crate::app::state::LoginEvent;
use crate::app::{AppEvent, BrowserEvent, ListStore};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/followed_artists.ui")]
    pub struct FollowedArtistsWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub flowbox: TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub status_page: TemplateChild<libadwaita::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FollowedArtistsWidget {
        const NAME: &'static str = "FollowedArtistsWidget";
        type Type = super::FollowedArtistsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FollowedArtistsWidget {}
    impl WidgetImpl for FollowedArtistsWidget {}
    impl BoxImpl for FollowedArtistsWidget {}
}

glib::wrapper! {
    pub struct FollowedArtistsWidget(ObjectSubclass<imp::FollowedArtistsWidget>) @extends gtk::Widget, gtk::Box;
}

impl FollowedArtistsWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp()
            .scrolled_window
            .connect_edge_reached(move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    f()
                }
            });
    }

    fn bind_artists<F>(&self, worker: Worker, store: &ListStore<ArtistModel>, on_artist_pressed: F)
    where
        F: Fn(String) + Clone + 'static,
    {
        self.imp()
            .flowbox
            .bind_model(Some(store.unsafe_store()), move |item| {
                let artist_model = item.downcast_ref::<ArtistModel>().unwrap();
                let child = gtk::FlowBoxChild::new();
                let artist = ArtistWidget::for_model(artist_model, worker.clone());

                let f = on_artist_pressed.clone();
                artist.connect_artist_pressed(clone!(@weak artist_model => move |_| {
                    f(artist_model.id());
                }));

                child.set_child(Some(&artist));
                child.upcast::<gtk::Widget>()
            });
    }
    pub fn get_status_page(&self) -> &libadwaita::StatusPage {
        &self.imp().status_page
    }
}

pub struct FollowedArtists {
    widget: FollowedArtistsWidget,
    worker: Worker,
    model: Rc<FollowedArtistsModel>,
}

impl FollowedArtists {
    pub fn new(worker: Worker, model: FollowedArtistsModel) -> Self {
        let model = Rc::new(model);

        let widget = FollowedArtistsWidget::new();

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more_followed_artists();
        }));

        Self {
            widget,
            worker,
            model,
        }
    }

    fn bind_flowbox(&self) {
        self.widget.bind_artists(
            self.worker.clone(),
            &self.model.get_list_store().unwrap(),
            clone!(@weak self.model as model => move |id| {
                model.open_artist(id);
            }),
        );
    }
}

impl EventListener for FollowedArtists {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                let _ = self.model.refresh_followed_artists();
                self.bind_flowbox();
            }
            AppEvent::LoginEvent(LoginEvent::LoginCompleted(_)) => {
                let _ = self.model.refresh_followed_artists();
            }
            AppEvent::BrowserEvent(BrowserEvent::FollowedArtistsUpdated) => {
                self.widget
                    .get_status_page()
                    .set_visible(!self.model.has_followed_artists());
            }
            _ => {}
        }
    }
}

impl Component for FollowedArtists {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.as_ref()
    }
}
