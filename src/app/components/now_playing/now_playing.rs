use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::NowPlayingModel;
use crate::app::components::{
    Component, EventListener, HeaderBarComponent, HeaderBarWidget, Playlist,
};
use crate::app::state::{Device, LoginEvent};
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
        pub title: TemplateChild<gtk::MenuButton>,

        #[template_child]
        pub popover: TemplateChild<gtk::Popover>,

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

    impl ObjectImpl for NowPlayingWidget {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

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
        self.widget().headerbar.as_ref()
    }

    fn make_devices_list(devices: &Vec<Device>) -> gtk::Box {
        let _box = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .build();
        let mut first_device: Option<gtk::CheckButton> = None;
        for device in devices {
            let check = gtk::CheckButtonBuilder::new().label(device.label());
            let check = if let Some(group) = first_device.as_ref() {
                check.group(group)
            } else {
                check
            }
            .build();
            _box.append(&check);
            first_device = first_device.or(Some(check));
        }
        _box
    }

    fn set_available_devices(&self, devices: &Vec<Device>) {
        let widget = self.widget();

        if devices.len() > 1 {
            let _box = Self::make_devices_list(devices);
            widget.popover.unparent();
            widget.popover.set_parent(&*widget.title);
            widget.title.set_popover(Some(&*widget.popover));
            widget.popover.set_child(Some(&_box));
        }
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
                self.widget
                    .set_available_devices(&*self.model.get_available_devices());
            }
            _ => (),
        }
        self.broadcast_event(event);
    }
}
