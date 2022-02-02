use gio::{SimpleAction, SimpleActionGroup};
use glib::FromVariant;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::NowPlayingModel;
use crate::app::components::{
    Component, EventListener, HeaderBarComponent, HeaderBarWidget, Playlist,
};
use crate::app::models::ConnectDevice;
use crate::app::state::{Device, LoginEvent};
use crate::app::{state::PlaybackEvent, AppEvent, Worker};

const ACTIONS: &str = "devices";
const CONNECT_ACTION: &str = "connect";

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
        pub popover: TemplateChild<gtk::PopoverMenu>,

        #[template_child]
        pub custom_content: TemplateChild<gtk::Box>,

        #[template_child]
        pub devices: TemplateChild<gtk::Box>,

        #[template_child]
        pub this_device_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub menu: TemplateChild<gio::MenuModel>,

        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        pub action_group: SimpleActionGroup,
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

            self.popover.set_menu_model(Some(&*self.menu));
            self.popover
                .add_child(&*self.custom_content, "custom_content");

            self.title.set_popover(Some(&*self.popover));
            self.title
                .upcast_ref::<gtk::Widget>()
                .insert_action_group(ACTIONS, Some(&self.action_group));

            self.this_device_button
                .set_action_name(Some(&format!("{}.{}", ACTIONS, CONNECT_ACTION)));
            self.this_device_button
                .set_action_target_value(Some(&Option::<String>::None.to_variant()));
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

    fn connect_refresh<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().action_group.add_action(&{
            let logout = SimpleAction::new("refresh", None);
            logout.connect_activate(move |_, _| f());
            logout
        });
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

    fn connect_switch_device<F>(&self, f: F)
    where
        F: Fn(Option<String>) + 'static,
    {
        self.imp().action_group.add_action(&{
            let logout = SimpleAction::new_stateful(
                "connect",
                Some(Option::<String>::static_variant_type().as_ref()),
                &Option::<String>::None.to_variant(),
            );
            logout.connect_activate(move |action, device_id| {
                if let Some(device_id) = device_id {
                    action.change_state(device_id);
                    f(Option::<String>::from_variant(device_id).unwrap());
                }
            });
            logout
        });
    }

    fn song_list_widget(&self) -> &gtk::ListView {
        self.imp().song_list.as_ref()
    }

    fn headerbar_widget(&self) -> &HeaderBarWidget {
        self.widget().headerbar.as_ref()
    }

    fn update_devices_list(&self, devices: &Vec<ConnectDevice>, active_device: &Device) {
        let widget = self.imp();
        widget.title.set_popover(Option::<&gtk::Widget>::None);
        widget.this_device_button.set_sensitive(!devices.is_empty());
        while let Some(child) = widget.devices.upcast_ref::<gtk::Widget>().first_child() {
            widget.devices.remove(&child);
        }
        for device in devices {
            let check = gtk::CheckButton::builder()
                .action_name(&format!("{}.{}", ACTIONS, CONNECT_ACTION))
                .action_target(&Some(&device.id).to_variant())
                .group(&*widget.this_device_button)
                .label(&device.label)
                .build();
            widget.devices.append(&check);
        }
        widget.title.set_popover(Some(&*widget.popover));
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

        widget.connect_refresh(clone!(@weak model => move || {
            model.refresh_available_devices();
        }));

        widget.connect_switch_device(clone!(@weak model => move |id| {
            model.set_current_device(id);
        }));

        let playlist = Box::new(Playlist::new(
            widget.song_list_widget().clone(),
            model.clone(),
            worker,
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
                self.widget.update_devices_list(
                    &*self.model.get_available_devices(),
                    &*self.model.get_current_device(),
                );
            }
            _ => (),
        }
        self.broadcast_event(event);
    }
}
