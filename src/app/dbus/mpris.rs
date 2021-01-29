#![allow(non_snake_case)]

use futures::channel::mpsc::Sender;
use std::convert::Into;
use zbus::dbus_interface;
use zbus::fdo::{Error, Result};
use zbus::ObjectServer;

use super::types::*;
use crate::app::AppAction;

pub struct SpotMpris;

#[derive(Clone)]
pub struct SpotMprisPlayer {
    pub state: SharedMprisState,
    sender: Sender<AppAction>,
}

impl SpotMprisPlayer {
    pub fn new(state: SharedMprisState, sender: Sender<AppAction>) -> Self {
        Self { state, sender }
    }
}

#[dbus_interface(interface = "org.mpris.MediaPlayer2")]
impl SpotMpris {
    fn quit(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    fn raise(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
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

#[dbus_interface(interface = "org.mpris.MediaPlayer2.Player")]
impl SpotMprisPlayer {
    pub fn next(&self) -> Result<()> {
        self.sender
            .clone()
            .try_send(AppAction::Next)
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    pub fn open_uri(&self, Uri: &str) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    pub fn pause(&self) -> Result<()> {
        // self.sender.clone().try_send(AppAction::TogglePlay)?;
        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        // self.sender.clone().try_send(AppAction::TogglePlay)?;
        Ok(())
    }

    pub fn play_pause(&self) -> Result<()> {
        self.sender
            .clone()
            .try_send(AppAction::TogglePlay)
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }


    // #[dbus_interface(signal_changed)]
    // pub fn emit_$prop_changed() -> zbus::Result<()>;

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

    pub fn notify_metadata(&self) -> zbus::Result<()> {
        let invalidated: Vec<String> = vec![];
        let mut changed = std::collections::HashMap::new();
        changed.insert(
            "Metadata",
            zvariant::Value::from(self.metadata()),
        );
        ObjectServer::local_node_emit_signal(
            None,
            "org.freedesktop.DBus.Properties",
            "PropertiesChanged",
            &("org.mpris.MediaPlayer2.Player", changed, invalidated),
        )
    }

    fn previous(&self) -> Result<()> {
        self.sender
            .clone()
            .try_send(AppAction::Previous)
            .map_err(|_| zbus::fdo::Error::Failed("Could not send action".to_string()))
    }

    pub fn seek(&self, Offset: i64) -> Result<()> {
        Ok(())
    }

    fn set_position(&self, TrackId: String, Position: i64) -> Result<()> {
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    #[dbus_interface(signal)]
    fn seeked(&self, Position: i64) -> zbus::Result<()>;

    #[dbus_interface(property)]
    pub fn can_control(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    pub fn can_go_next(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    pub fn can_go_previous(&self) -> bool {
        true
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
        false
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
        0i64
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
