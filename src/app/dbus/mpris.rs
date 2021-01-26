#![allow(non_snake_case)] 

use futures::channel::mpsc::Sender;
use zbus::dbus_interface;
use zbus::fdo::{Error, Result};

use crate::app::AppAction;

pub struct SpotMpris(pub Sender<AppAction>);
pub struct SpotMprisPlayer(pub Sender<AppAction>);


#[dbus_interface(interface = "org.mpris.MediaPlayer2")]
impl SpotMpris {
    /// Quit method
    fn quit(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// Raise method
    fn raise(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// CanQuit property
    #[dbus_interface(property)]
    fn can_quit(&self) -> bool {
        false
    }

    /// CanRaise property
    #[dbus_interface(property)]
    fn can_raise(&self) -> bool {
        true
    }

    /// HasTrackList property
    #[dbus_interface(property)]
    fn has_track_list(&self) -> bool {
        false
    }

    /// Identity property
    #[dbus_interface(property)]
    fn identity(&self) -> String {
        "Spot".to_string()
    }

    /// SupportedMimeTypes property
    #[dbus_interface(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        vec![]
    }

    /// SupportedUriSchemes property
    #[dbus_interface(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        vec![]
    }
}

#[dbus_interface(interface = "org.mpris.MediaPlayer2.Player")]
impl SpotMprisPlayer {
    /// Next method
    fn next(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// OpenUri method
    fn open_uri(&self, Uri: &str) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// Pause method
    fn pause(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// Play method
    fn play(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// PlayPause method
    fn play_pause(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// Previous method
    fn previous(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// Seek method
    fn seek(&self, Offset: i64) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// SetPosition method
    fn set_position(&self, TrackId: String, Position: i64) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// Stop method
    fn stop(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    /// Seeked signal
    #[dbus_interface(signal)]
    fn seeked(&self, Position: i64) -> zbus::Result<()>;

    /// CanControl property
    #[dbus_interface(property)]
    fn can_control(&self) -> bool {
        true
    }

    /// CanGoNext property
    #[dbus_interface(property)]
    fn can_go_next(&self) -> bool {
        false
    }

    /// CanGoPrevious property
    #[dbus_interface(property)]
    fn can_go_previous(&self) -> bool {
        false
    }

    /// CanPause property
    #[dbus_interface(property)]
    fn can_pause(&self) -> bool {
        false
    }

    /// CanPlay property
    #[dbus_interface(property)]
    fn can_play(&self) -> bool {
        true
    }

    /// CanSeek property
    #[dbus_interface(property)]
    fn can_seek(&self) -> bool {
        false
    }

    /// MaximumRate property
    #[dbus_interface(property)]
    fn maximum_rate(&self) -> f64 {
        1.0f64
    }

    /// Metadata property
    #[dbus_interface(property)]
    fn metadata(&self) -> std::collections::HashMap<&str, zvariant::Value> {
        let mut meta = std::collections::HashMap::new();
        meta.insert("xesam:albumArtist", "test".into());
        meta.insert("xesam:title", "test".into());
        meta
    }

    /// MinimumRate property
    #[dbus_interface(property)]
    fn minimum_rate(&self) -> f64 {
        1.0f64
    }

    /// PlaybackStatus property
    #[dbus_interface(property)]
    fn playback_status(&self) -> &'static str {
        "Playing"
    }

    /// Position property
    #[dbus_interface(property)]
    fn position(&self) -> i64 {
        0i64
    }

    /// Rate property
    #[dbus_interface(property)]
    fn rate(&self) -> f64 {
        1.0f64
    }

    #[dbus_interface(property)]
    fn set_rate(&self, value: f64) {}

    /// Shuffle property
    #[dbus_interface(property)]
    fn shuffle(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn set_shuffle(&self, value: bool) {}

    /// Volume property
    #[dbus_interface(property)]
    fn volume(&self) -> f64 {
        0f64
    }

    #[dbus_interface(property)]
    fn set_volume(&self, value: f64) {}
}