#![allow(non_snake_case)]

use futures::channel::mpsc::Sender;
use zbus::{dbus_interface, Interface};
use zbus::fdo::{Error, Result};
use zvariant::{Dict, Str, Signature, Value};
use std::convert::Into;
use zvariant::Type;

use crate::app::AppAction;

fn boxed_value<'a, V: Into<Value<'a>>>(v: V) -> Value<'a> {
    Value::new(v.into())
}

#[derive(Clone, Copy, Debug)]
enum PlaybackStatus {
    Playing,
    Paused,
    Stopped
}

impl Type for PlaybackStatus {
    fn signature() -> Signature<'static> {
        Str::signature()
    }
}

impl From<PlaybackStatus> for Value<'_> {

    fn from(status: PlaybackStatus) -> Self {
        match status {
            PlaybackStatus::Playing => "Playing".into(),
            PlaybackStatus::Paused => "Paused".into(),
            PlaybackStatus::Stopped => "Stopped".into()
        }
    }
}

struct TrackMetadata {
    artist: Vec<String>,
    title: String
}

impl Type for TrackMetadata {
    fn signature() -> Signature<'static> {
        Signature::from_str_unchecked("a{sv}")
    }
}

impl <'a> From<&'a TrackMetadata> for Value<'a> {

    fn from(meta: &'a TrackMetadata) -> Self {
        let mut d = Dict::new(Str::signature(), Value::signature());
        d.append("xesam:title".into(), boxed_value(&meta.title)).unwrap();
        d.append("xesam:artist".into(), boxed_value(&meta.artist)).unwrap();
        d.append("xesam:albumArtist".into(), boxed_value(&meta.artist)).unwrap();
        Value::Dict(d)
    }
}


pub struct SpotMpris(pub Sender<AppAction>);

pub struct SpotMprisPlayer {
    track: TrackMetadata,
    status: PlaybackStatus
}

impl SpotMprisPlayer {

    pub fn new() -> Self {
        Self {
            track: TrackMetadata {
                title: "test".to_string(),
                artist: vec!["test2".to_string()],
            },
            status: PlaybackStatus::Paused
        }
    }
}

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
    fn identity(&self) -> &'static str {
        "Spot"
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
    fn play(&mut self) -> Result<()> {
        self.status = PlaybackStatus::Playing;
        Ok(())
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
    fn metadata(&self) -> &TrackMetadata {
        &self.track
    }

    /// MinimumRate property
    #[dbus_interface(property)]
    fn minimum_rate(&self) -> f64 {
        1.0f64
    }

    /// PlaybackStatus property
    #[dbus_interface(property)]
    fn playback_status(&self) -> PlaybackStatus {
        self.status
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
