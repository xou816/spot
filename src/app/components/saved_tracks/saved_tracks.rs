use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use crate::app::components::{display_add_css_provider, Component, EventListener, Playlist};
use crate::app::state::LoginEvent;
use crate::app::{AppEvent, Worker};

use super::SavedTracksModel;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/saved_tracks.ui")]
    pub struct SavedTracksWidget {
        #[template_child]
        pub song_list: TemplateChild<gtk::ListView>,

        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SavedTracksWidget {
        const NAME: &'static str = "SavedTracksWidget";
        type Type = super::SavedTracksWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SavedTracksWidget {}
    impl WidgetImpl for SavedTracksWidget {}
    impl BoxImpl for SavedTracksWidget {}
}

glib::wrapper! {
    pub struct SavedTracksWidget(ObjectSubclass<imp::SavedTracksWidget>) @extends gtk::Widget, gtk::Box;
}

impl SavedTracksWidget {
    fn new() -> Self {
        display_add_css_provider(resource!("/components/saved_tracks.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of SavedTracksWidget")
    }

    fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::SavedTracksWidget::from_instance(self)
            .scrolled_window
            .connect_edge_reached(move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    f()
                }
            });
    }

    fn song_list_widget(&self) -> &gtk::ListView {
        imp::SavedTracksWidget::from_instance(self)
            .song_list
            .as_ref()
    }
}

pub struct SavedTracks {
    widget: SavedTracksWidget,
    model: Rc<SavedTracksModel>,
    children: Vec<Box<dyn EventListener>>,
}

impl SavedTracks {
    pub fn new(model: Rc<SavedTracksModel>, worker: Worker) -> Self {
        let widget = SavedTracksWidget::new();

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more();
        }));

        let playlist = Playlist::new(
            widget.song_list_widget().clone(),
            model.clone(),
            worker,
            true,
        );

        Self {
            widget,
            model,
            children: vec![Box::new(playlist)],
        }
    }
}

impl Component for SavedTracks {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for SavedTracks {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started | AppEvent::LoginEvent(LoginEvent::LoginCompleted(_)) => {
                self.model.load_initial();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
