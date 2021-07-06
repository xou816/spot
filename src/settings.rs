use crate::player::{AudioBackend, SpotifyPlayerSettings};
use gio::prelude::SettingsExt;
use librespot::playback::config::Bitrate;

const SETTINGS: &str = "dev.alextren.Spot";

#[derive(Clone)]
pub struct WindowGeometry {
    pub width: i32,
    pub height: i32,
    pub is_maximized: bool,
}

impl WindowGeometry {
    pub fn new_from_gsettings() -> Self {
        let settings = gio::Settings::new(SETTINGS);
        Self {
            width: settings.int("window-width"),
            height: settings.int("window-height"),
            is_maximized: settings.boolean("window-is-maximized"),
        }
    }

    pub fn save(&self) -> Option<()> {
        let settings = gio::Settings::new(SETTINGS);
        settings.delay();
        settings.set_int("window-width", self.width).ok()?;
        settings.set_int("window-height", self.height).ok()?;
        settings
            .set_boolean("window-is-maximized", self.is_maximized)
            .ok()?;
        settings.apply();
        Some(())
    }
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            is_maximized: false,
        }
    }
}

impl SpotifyPlayerSettings {
    pub fn new_from_gsettings() -> Option<Self> {
        let settings = gio::Settings::new(SETTINGS);
        let bitrate = match settings.enum_("player-bitrate") {
            0 => Some(Bitrate::Bitrate96),
            1 => Some(Bitrate::Bitrate160),
            2 => Some(Bitrate::Bitrate320),
            _ => None,
        }?;
        let backend = match settings.enum_("audio-backend") {
            0 => Some(AudioBackend::PulseAudio),
            1 => Some(AudioBackend::Alsa(
                settings.string("alsa-device").as_str().to_string(),
            )),
            _ => None,
        }?;
        Some(Self { bitrate, backend })
    }
}

pub struct SpotSettings {
    pub prefers_dark_theme: bool,
    pub player_settings: SpotifyPlayerSettings,
    pub window: WindowGeometry,
}

impl SpotSettings {
    pub fn new_from_gsettings() -> Option<Self> {
        let settings = gio::Settings::new(SETTINGS);
        let prefers_dark_theme = settings.boolean("prefers-dark-theme");
        Some(Self {
            prefers_dark_theme,
            player_settings: SpotifyPlayerSettings::new_from_gsettings()?,
            window: WindowGeometry::new_from_gsettings(),
        })
    }
}

impl Default for SpotSettings {
    fn default() -> Self {
        Self {
            prefers_dark_theme: true,
            player_settings: Default::default(),
            window: Default::default(),
        }
    }
}
