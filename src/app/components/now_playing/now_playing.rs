use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::NowPlayingModel;
use crate::app::components::{
    Component, EventListener, HeaderBarComponent, HeaderBarWidget, Playlist,
};
use crate::app::state::LoginEvent;
use crate::app::{state::PlaybackEvent, AppEvent, Worker};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/now_playing.ui")]
    pub struct NowPlayingWidget {
        #[template_child]
        pub song_list: TemplateChild<gtk::ListView>,

        #[template_child]
        pub headerbar: TemplateChild<HeaderBarWidget>,

        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NowPlayingWidget {
        const NAME: &'static str = "NowPlayingWidget";
        type Type = super::NowPlayingWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NowPlayingWidget {}
    impl WidgetImpl for NowPlayingWidget {}
    impl BoxImpl for NowPlayingWidget {}
}

glib::wrapper! {
    pub struct NowPlayingWidget(ObjectSubclass<imp::NowPlayingWidget>) @extends gtk::Widget, gtk::Box;
}

impl NowPlayingWidget {
    fn new() -> Self {
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

    fn song_list_widget(&self) -> &gtk::ListView {
        self.imp().song_list.as_ref()
    }

    fn headerbar_widget(&self) -> &HeaderBarWidget {
        imp::NowPlayingWidget::from_instance(self)
            .headerbar
            .as_ref()
    }
}

pub struct NowPlaying {
    widget: NowPlayingWidget,
    model: Rc<NowPlayingModel>,
    children: Vec<Box<dyn EventListener>>,
}

impl NowPlaying {
    pub fn new(model: Rc<NowPlayingModel>, worker: Worker) -> Self {
        let widget = NowPlayingWidget::new();

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more();
        }));

        let playlist = Box::new(Playlist::new(
            widget.song_list_widget().clone(),
            model.clone(),
            worker
        ));

        let headerbar = Box::new(HeaderBarComponent::new(
            widget.headerbar_widget().clone(),
            model.to_headerbar_model(),
        ));

        Self {
            widget,
            model,
            children: vec![playlist, headerbar],
        }
    }
}

impl Component for NowPlaying {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for NowPlaying {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) => {
                self.model.load_more();
            }
            AppEvent::LoginEvent(LoginEvent::LoginCompleted(_)) => {
                self.model.refresh_available_devices();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::AvailableDevicesChanged) => {
                dbg!(&*self.model.get_available_devices());
            }
            _ => (),
        }
        self.broadcast_event(event);
    }
}
