#![allow(non_snake_case)]
#![allow(unused_variables)]

use futures::channel::mpsc::UnboundedSender;
use std::convert::{Into, TryInto};
use zbus::dbus_interface;
use zbus::fdo::{Error, Result};
use zbus::ObjectServer;
use zvariant::ObjectPath;

use super::types::*;
use crate::app::{state::PlaybackAction, AppAction};

#[derive(Clone)]
pub struct SpotMpris {
    sender: UnboundedSender<AppAction>,
}

impl SpotMpris {
    pub fn new(sender: UnboundedSender<AppAction>) -> Self {
        Self { sender }
    }
}

#[dbus_interface(interface = "org.mpris.MediaPlayer2")]
impl SpotMpris {
    fn quit(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    fn raise(&self) -> Result<()> {
        self.sender
            .unbounded_send(AppAction::Raise)
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    #[dbus_interface(property)]
    fn can_quit(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn can_raise(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn has_track_list(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn identity(&self) -> &'static str {
        "Spot"
    }

    #[dbus_interface(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        vec![]
    }

    #[dbus_interface(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        vec![]
    }
}

#[derive(Clone)]
pub struct SpotMprisPlayer {
    pub state: SharedMprisState,
    sender: UnboundedSender<AppAction>,
}

impl SpotMprisPlayer {
    pub fn new(state: SharedMprisState, sender: UnboundedSender<AppAction>) -> Self {
        Self { state, sender }
    }
}

#[dbus_interface(interface = "org.mpris.MediaPlayer2.Player")]
impl SpotMprisPlayer {
    pub fn next(&mut self) -> Result<()> {
        self.sender
            .unbounded_send(PlaybackAction::Next.into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    pub fn open_uri(&self, Uri: &str) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    pub fn pause(&self) -> Result<()> {
        self.sender
            .unbounded_send(PlaybackAction::Pause.into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    pub fn play(&self) -> Result<()> {
        self.sender
            .unbounded_send(PlaybackAction::Play.into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    pub fn play_pause(&mut self) -> Result<()> {
        self.sender
            .unbounded_send(PlaybackAction::TogglePlay.into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    pub fn notify_playback_status(&self) -> zbus::Result<()> {
        let invalidated: Vec<String> = vec![];
        let mut changed = std::collections::HashMap::new();
        changed.insert(
            "PlaybackStatus",
            zvariant::Value::from(self.playback_status()),
        );
        ObjectServer::local_node_emit_signal(
            None,
            "org.freedesktop.DBus.Properties",
            "PropertiesChanged",
            &("org.mpris.MediaPlayer2.Player", changed, invalidated),
        )
    }

    pub fn notify_metadata_and_prev_next(&self) -> zbus::Result<()> {
        let invalidated: Vec<String> = vec![];
        let mut changed = std::collections::HashMap::new();
        changed.insert("Metadata", zvariant::Value::from(self.metadata()));
        changed.insert("CanGoNext", zvariant::Value::from(self.can_go_next()));
        changed.insert(
            "CanGoPrevious",
            zvariant::Value::from(self.can_go_previous()),
        );
        ObjectServer::local_node_emit_signal(
            None,
            "org.freedesktop.DBus.Properties",
            "PropertiesChanged",
            &("org.mpris.MediaPlayer2.Player", changed, invalidated),
        )
    }

    fn previous(&mut self) -> Result<()> {
        self.sender
            .unbounded_send(PlaybackAction::Previous.into())
            .map_err(|_| zbus::fdo::Error::Failed("Could not send action".to_string()))
    }

    pub fn seek(&self, Offset: i64) -> Result<()> {
        if !self.state.current_track().is_some() {
            return Ok(());
        }

        let pos: u32 = (self.state.position() / 1000)
            .try_into()
            .map_err(|_| zbus::fdo::Error::Failed("Could not parse position".to_string()))?;

        let offset: u32 = (Offset / 1000)
            .try_into()
            .map_err(|_| zbus::fdo::Error::Failed("Could not parse position".to_string()))?;

        self.sender
            .unbounded_send(PlaybackAction::Seek(pos + offset).into())
            .map_err(|_| zbus::fdo::Error::Failed("Could not send action".to_string()))
    }

    pub fn set_position(&self, TrackId: ObjectPath, Position: i64) -> Result<()> {
        if !self.state.current_track().is_some() {
            return Ok(());
        }

        if TrackId.to_string() != self.metadata().id {
            return Ok(());
        }

        let length: i64 = self.metadata().length.try_into().map_err(|_| {
            zbus::fdo::Error::Failed("Could not cast length (too large)".to_string())
        })?;

        if Position > length {
            return Ok(());
        }

        let pos: u32 = (Position / 1000)
            .try_into()
            .map_err(|_| zbus::fdo::Error::Failed("Could not parse position".to_string()))?;

        self.sender
            .unbounded_send(PlaybackAction::Seek(pos).into())
            .map_err(|_| zbus::fdo::Error::Failed("Could not send action".to_string()))
    }

    pub fn stop(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    #[dbus_interface(signal)]
    #[rustfmt::skip]
    pub fn seeked(&self, Position: i64) -> zbus::Result<()>;

    #[dbus_interface(property)]
    pub fn can_control(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    pub fn can_go_next(&self) -> bool {
        self.state.has_next()
    }

    #[dbus_interface(property)]
    pub fn can_go_previous(&self) -> bool {
        self.state.has_prev()
    }

    #[dbus_interface(property)]
    pub fn can_pause(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    pub fn can_play(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    pub fn can_seek(&self) -> bool {
        self.state.current_track().is_some()
    }

    #[dbus_interface(property)]
    pub fn maximum_rate(&self) -> f64 {
        1.0f64
    }

    #[dbus_interface(property)]
    pub fn metadata(&self) -> TrackMetadata {
        self.state.current_track().unwrap_or(TrackMetadata {
            id: String::new(),
            length: 0,
            title: "Not playing".to_string(),
            artist: vec![],
            album: String::new(),
            art: None,
        })
    }

    #[dbus_interface(property)]
    pub fn minimum_rate(&self) -> f64 {
        1.0f64
    }

    #[dbus_interface(property)]
    pub fn playback_status(&self) -> PlaybackStatus {
        self.state.status()
    }

    #[dbus_interface(property)]
    pub fn position(&self) -> i64 {
        self.state.position() as i64
    }

    #[dbus_interface(property)]
    pub fn rate(&self) -> f64 {
        1.0f64
    }

    #[dbus_interface(property)]
    pub fn set_rate(&self, value: f64) {}

    #[dbus_interface(property)]
    pub fn shuffle(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    pub fn set_shuffle(&self, value: bool) {}

    #[dbus_interface(property)]
    pub fn volume(&self) -> f64 {
        0f64
    }

    #[dbus_interface(property)]
    pub fn set_volume(&self, value: f64) {}
}
