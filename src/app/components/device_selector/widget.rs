use crate::app::models::{ConnectDevice, ConnectDeviceKind};
use crate::app::state::Device;
use gettextrs::gettext;
use gio::{Action, SimpleAction, SimpleActionGroup};
use glib::FromVariant;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

const ACTIONS: &str = "devices";
const CONNECT_ACTION: &str = "connect";
const REFRESH_ACTION: &str = "refresh";

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/device_selector.ui")]
    pub struct DeviceSelectorWidget {
        #[template_child]
        pub button_content: TemplateChild<libadwaita::ButtonContent>,

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

        pub action_group: SimpleActionGroup,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DeviceSelectorWidget {
        const NAME: &'static str = "DeviceSelectorWidget";
        type Type = super::DeviceSelectorWidget;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DeviceSelectorWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let popover: &gtk::PopoverMenu = &self.popover;
            popover.set_menu_model(Some(&*self.menu));
            popover.add_child(&*self.custom_content, "custom_content");
            popover.set_parent(&*self.obj());
            popover.set_autohide(true);

            let this_device = &*self.this_device_button;
            this_device.set_action_name(Some(&format!("{}.{}", ACTIONS, CONNECT_ACTION)));
            this_device.set_action_target_value(Some(&Option::<String>::None.to_variant()));

            self.obj()
                .insert_action_group(ACTIONS, Some(&self.action_group));
            self.obj()
                .connect_clicked(clone!(@weak popover => move |_| {
                    popover.set_visible(true);
                    popover.present();
                    popover.grab_focus();
                }));
        }
    }

    impl WidgetImpl for DeviceSelectorWidget {}
    impl ButtonImpl for DeviceSelectorWidget {}
}

glib::wrapper! {
    pub struct DeviceSelectorWidget(ObjectSubclass<imp::DeviceSelectorWidget>) @extends gtk::Widget, gtk::Button;
}

impl DeviceSelectorWidget {
    fn action(&self, name: &str) -> Option<Action> {
        self.imp().action_group.lookup_action(name)
    }

    pub fn connect_refresh<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().action_group.add_action(&{
            let refresh = SimpleAction::new(REFRESH_ACTION, None);
            refresh.connect_activate(move |_, _| f());
            refresh
        });
    }

    pub fn connect_switch_device<F>(&self, f: F)
    where
        F: Fn(Option<String>) + 'static,
    {
        self.imp().action_group.add_action(&{
            let connect = SimpleAction::new_stateful(
                CONNECT_ACTION,
                Some(Option::<String>::static_variant_type().as_ref()),
                Option::<String>::None.to_variant(),
            );
            connect.connect_activate(move |action, device_id| {
                if let Some(device_id) = device_id {
                    action.change_state(device_id);
                    f(Option::<String>::from_variant(device_id).unwrap());
                }
            });
            connect
        });
    }

    pub fn set_current_device(&self, device: &Device) {
        if let Some(action) = self.action(CONNECT_ACTION) {
            let device_id = match device {
                Device::Local => None,
                Device::Connect(connect) => Some(&connect.id),
            };
            action.change_state(&device_id.to_variant());
        }
        let label = match device {
            Device::Local => gettext("This device"),
            Device::Connect(connect) => connect.label.clone(),
        };
        let icon = match device {
            Device::Local => "audio-x-generic-symbolic",
            Device::Connect(connect) => match connect.kind {
                ConnectDeviceKind::Phone => "phone-symbolic",
                ConnectDeviceKind::Computer => "computer-symbolic",
                ConnectDeviceKind::Speaker => "audio-speakers-symbolic",
                ConnectDeviceKind::Other => "audio-x-generic-symbolic",
            },
        };
        self.imp().button_content.set_label(&label);
        self.imp().button_content.set_icon_name(icon);
    }

    pub fn update_devices_list(&self, devices: &[ConnectDevice]) {
        let widget = self.imp();
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
    }
}
