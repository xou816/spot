use crate::player::{AudioBackend, SpotifyPlayerSettings};
use gio::SettingsExt;
use librespot::playback::config::Bitrate;

pub struct SpotSettings {
    pub prefers_dark_theme: bool,
    pub player_settings: SpotifyPlayerSettings,
}

impl SpotSettings {
    pub fn new_or_default() -> Self {
        Self::new().unwrap_or_else(|| Self {
            prefers_dark_theme: true,
            player_settings: SpotifyPlayerSettings {
                bitrate: Bitrate::Bitrate160,
                backend: AudioBackend::PulseAudio,
            },
        })
    }

    pub fn new() -> Option<Self> {
        let settings = gio::Settings::new("dev.alextren.Spot");
        let prefers_dark_theme = settings.get_boolean("prefers-dark-theme");
        let bitrate = match settings.get_enum("player-bitrate") {
            0 => Some(Bitrate::Bitrate96),
            1 => Some(Bitrate::Bitrate160),
            2 => Some(Bitrate::Bitrate320),
            _ => None,
        };
        let backend = match settings.get_enum("audio-backend") {
            0 => Some(AudioBackend::PulseAudio),
            1 => Some(AudioBackend::Alsa(
                settings.get_string("alsa-device")?.as_str().to_string(),
            )),
            _ => None,
        };
        let player_settings = SpotifyPlayerSettings {
            bitrate: bitrate?,
            backend: backend?,
        };
        Some(Self {
            prefers_dark_theme,
            player_settings,
        })
    }
}
