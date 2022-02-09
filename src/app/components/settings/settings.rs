use crate::app::components::{display_add_css_provider, EventListener};
use crate::app::AppEvent;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use super::SettingsModel;

mod imp {

    use super::*;
    use libadwaita::prelude::*;
    use libadwaita::subclass::prelude::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/settings.ui")]
    pub struct SettingsWindow {
        #[template_child]
        pub player_bitrate: TemplateChild<libadwaita::ComboRow>,

        #[template_child]
        pub alsa_backend: TemplateChild<gtk::Entry>,

        #[template_child]
        pub audio_backend: TemplateChild<libadwaita::ComboRow>,

        #[template_child]
        pub ap_port: TemplateChild<gtk::Entry>,

        #[template_child]
        pub theme: TemplateChild<libadwaita::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsWindow {
        const NAME: &'static str = "SettingsWindow";
        type Type = super::SettingsWindow;
        type ParentType = libadwaita::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsWindow {}
    impl WidgetImpl for SettingsWindow {}
    impl WindowImpl for SettingsWindow {}
    impl AdwWindowImpl for SettingsWindow {}
    impl PreferencesWindowImpl for SettingsWindow {}
}

glib::wrapper! {
    pub struct SettingsWindow(ObjectSubclass<imp::SettingsWindow>) @extends gtk::Widget, libadwaita::PreferencesWindow;
}

impl SettingsWindow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of SettingsWindow")
    }

    pub fn show_self(&self) {}
}

pub struct Settings {
    parent: gtk::Window,
    settings_window: SettingsWindow,
    model: SettingsModel,
}

impl Settings {
    pub fn new(parent: gtk::Window, model: SettingsModel) -> Self {
        let settings_window = SettingsWindow::new();

        Self {
            parent,
            model,
            settings_window,
        }
    }

    fn window(&self) -> &libadwaita::Window {
        let pref_window = self
            .settings_window
            .upcast_ref::<libadwaita::PreferencesWindow>();
        pref_window.upcast_ref::<libadwaita::Window>()
    }

    pub fn show_self(&self) {
        self.window().set_transient_for(Some(&self.parent));
        self.window().set_modal(true);
        self.window().show();
    }
}

impl EventListener for Settings {
    fn on_event(&mut self, _: &AppEvent) {}
}
