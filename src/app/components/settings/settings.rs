use std::cell::RefCell;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::AppEvent;
use crate::player::{AudioBackend, SpotifyPlayerSettings};
use crate::settings::{SpotSettings, WindowGeometry};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use libadwaita::prelude::*;
use librespot::playback::config::Bitrate;

use super::SettingsModel;

const SETTINGS: &str = "dev.alextren.Spot";

mod imp {

    use super::*;
    use libadwaita::subclass::prelude::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/settings.ui")]
    pub struct SettingsWindow {
        #[template_child]
        pub player_bitrate: TemplateChild<libadwaita::ComboRow>,

        #[template_child]
        pub alsa_device: TemplateChild<gtk::Entry>,

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
        let window: Self =
            glib::Object::new(&[]).expect("Failed to create an instance of SettingsWindow");
        window.bind_settings();
        window.connect_theme_select();
        window
    }

    fn bind_settings(&self) {
        let widget = imp::SettingsWindow::from_instance(self);
        let settings = gio::Settings::new(SETTINGS);

        let player_bitrate = widget
            .player_bitrate
            .downcast_ref::<libadwaita::ComboRow>()
            .unwrap();
        settings
            .bind("player-bitrate", player_bitrate, "selected")
            .mapping(|variant, _| {
                variant.str().map(|s| {
                    match s {
                        "96" => 0,
                        "160" => 1,
                        "320" => 2,
                        _ => unreachable!(),
                    }
                    .to_value()
                })
            })
            .set_mapping(|value, _| {
                value.get::<u32>().ok().map(|u| {
                    match u {
                        0 => "96",
                        1 => "160",
                        2 => "320",
                        _ => unreachable!(),
                    }
                    .to_variant()
                })
            })
            .build();

        let alsa_device = widget.alsa_device.downcast_ref::<gtk::Entry>().unwrap();
        settings.bind("alsa-device", alsa_device, "text").build();

        let audio_backend = widget
            .audio_backend
            .downcast_ref::<libadwaita::ComboRow>()
            .unwrap();
        settings
            .bind("audio-backend", audio_backend, "selected")
            .mapping(|variant, _| {
                variant.str().map(|s| {
                    match s {
                        "pulseaudio" => 0,
                        "alsa" => 1,
                        _ => unreachable!(),
                    }
                    .to_value()
                })
            })
            .set_mapping(|value, _| {
                value.get::<u32>().ok().map(|u| {
                    match u {
                        0 => "pulseaudio",
                        1 => "alsa",
                        _ => unreachable!(),
                    }
                    .to_variant()
                })
            })
            .build();

        let ap_port = widget.ap_port.downcast_ref::<gtk::Entry>().unwrap();
        settings
            .bind("ap-port", ap_port, "text")
            .mapping(|variant, _| variant.get::<u32>().map(|s| s.to_value()))
            .set_mapping(|value, _| value.get::<u32>().ok().map(|u| u.to_variant()))
            .build();

        let theme = widget.theme.downcast_ref::<libadwaita::ComboRow>().unwrap();
        settings
            .bind("prefers-dark-theme", theme, "selected")
            .mapping(|variant, _| {
                variant
                    .get::<bool>()
                    .map(|prefer_dark| if prefer_dark { 1 } else { 0 }.to_value())
            })
            .set_mapping(|value, _| value.get::<u32>().ok().map(|u| (u == 1).to_variant()))
            .build();
    }

    fn connect_theme_select(&self) {
        let widget = imp::SettingsWindow::from_instance(self);
        let theme = widget.theme.downcast_ref::<libadwaita::ComboRow>().unwrap();
        theme.connect_selected_notify(|theme| {
            let prefers_dark_theme = theme.selected() == 1;
            let manager = libadwaita::StyleManager::default();

            manager.set_color_scheme(if prefers_dark_theme {
                libadwaita::ColorScheme::PreferDark
            } else {
                libadwaita::ColorScheme::PreferLight
            });
        });
    }

    fn connect_close<F>(&self, on_close: F)
    where
        F: Fn(SpotSettings) + 'static,
    {
        let pref_window = self.upcast_ref::<libadwaita::PreferencesWindow>();
        let window = pref_window.upcast_ref::<libadwaita::Window>();

        window.connect_close_request(
            clone!(@weak self as _self => @default-return gtk::Inhibit(false), move |_| {
                let widget = imp::SettingsWindow::from_instance(&_self);

                let bitrate = match widget.player_bitrate.selected() {
                    0 => Bitrate::Bitrate96,
                    1 => Bitrate::Bitrate160,
                    2 => Bitrate::Bitrate320,
                    _ => unreachable!(),
                };
                let alsa_device = widget.alsa_device.text();
                let backend = match widget.audio_backend.selected() {
                    0 => AudioBackend::PulseAudio,
                    1 => AudioBackend::Alsa(alsa_device.as_str().into()),
                    _ => unreachable!(),
                };
                let ap_port = widget.ap_port.text().parse().ok().and_then(|port| match port {
                    0 => None,
                    port => Some(port),
                });
                let prefers_dark_theme = widget.theme.selected() == 1;

                let player_settings = SpotifyPlayerSettings {
                    bitrate,
                    backend,
                    ap_port,
                };
                let window_geometry = WindowGeometry::new_from_gsettings();
                let settings = SpotSettings {
                    prefers_dark_theme,
                    player_settings,
                    window: window_geometry,
                };

                on_close(settings);
                gtk::Inhibit(false)
            }),
        );
    }
}

pub struct Settings {
    parent: gtk::Window,
    settings_window: SettingsWindow,
    settings: Rc<RefCell<SpotSettings>>,
}

impl Settings {
    pub fn new(parent: gtk::Window, model: SettingsModel) -> Self {
        let settings_window = SettingsWindow::new();
        let settings = Rc::new(RefCell::new(
            SpotSettings::new_from_gsettings().unwrap_or_default(),
        ));

        settings_window.connect_close(clone!(@weak settings => move |new_settings| {
            if settings.borrow().player_settings != new_settings.player_settings {
                model.set_player_settings(new_settings.player_settings);
            }
        }));

        settings_window.connect_show(clone!(@weak settings => move |_| {
            settings.replace(SpotSettings::new_from_gsettings().unwrap_or_default());
        }));

        Self {
            parent,
            settings_window,
            settings,
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
