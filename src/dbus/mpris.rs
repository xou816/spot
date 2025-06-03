#![allow(non_snake_case)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::convert::TryInto;

use futures::channel::mpsc::UnboundedSender;
use zbus::fdo::{Error, Result};
use zbus::{dbus_interface, Interface, SignalContext};
use zvariant::{ObjectPath, Value};

use super::types::*;
use crate::app::models::RepeatMode;
use crate::app::state::PlaybackAction;
use crate::app::AppAction;

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

    #[dbus_interface(property)]
    fn desktop_entry(&self) -> &'static str {
        "dev.alextren.Spot"
    }
}

pub struct SpotMprisPlayer {
    state: MprisState,
    sender: UnboundedSender<AppAction>,
}

impl SpotMprisPlayer {
    pub fn new(sender: UnboundedSender<AppAction>) -> Self {
        Self {
            state: MprisState::new(),
            sender,
        }
    }

    pub fn state_mut(&mut self) -> &mut MprisState {
        &mut self.state
    }

    pub async fn notify_current_track_changed(&self, ctxt: &SignalContext<'_>) -> zbus::Result<()> {
        let metadata = Value::from(self.metadata());
        let can_go_next = Value::from(self.can_go_next());
        let can_go_previous = Value::from(self.can_go_previous());

        zbus::fdo::Properties::properties_changed(
            ctxt,
            Self::name(),
            &HashMap::from([
                ("Metadata", &metadata),
                ("CanGoNext", &can_go_next),
                ("CanGoPrevious", &can_go_previous),
            ]),
            &[],
        )
        .await
    }

    pub async fn notify_loop_status(&self, ctxt: &SignalContext<'_>) -> zbus::Result<()> {
        let loop_status = Value::from(self.loop_status());
        let can_go_next = Value::from(self.can_go_next());
        let can_go_previous = Value::from(self.can_go_previous());

        zbus::fdo::Properties::properties_changed(
            ctxt,
            Self::name(),
            &HashMap::from([
                ("LoopStatus", &loop_status),
                ("CanGoNext", &can_go_next),
                ("CanGoPrevious", &can_go_previous),
            ]),
            &[],
        )
        .await
    }
}

#[dbus_interface(interface = "org.mpris.MediaPlayer2.Player")]
impl SpotMprisPlayer {
    pub fn next(&self) -> Result<()> {
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

    pub fn play_pause(&self) -> Result<()> {
        self.sender
            .unbounded_send(PlaybackAction::TogglePlay.into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    pub fn previous(&self) -> Result<()> {
        self.sender
            .unbounded_send(PlaybackAction::Previous.into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    pub fn seek(&self, Offset: i64) -> Result<()> {
        if self.state.current_track().is_none() {
            return Ok(());
        }

        let mut new_pos: i128 = (self.state.position() as i128 + i128::from(Offset)) / 1000;
        // As per spec, if new position is less than 0 round to 0
        if new_pos < 0 {
            new_pos = 0;
        }

        let new_pos: u32 = (new_pos)
            .try_into()
            .map_err(|_| Error::Failed("Could not parse position".to_string()))?;

        // As per spec, if new position is past the length of the song skip to
        // the next song
        if u64::from(new_pos) >= self.metadata().length / 1000 {
            self.sender
                .unbounded_send(PlaybackAction::Next.into())
                .map_err(|_| Error::Failed("Could not send action".to_string()))
        } else {
            self.sender
                .unbounded_send(PlaybackAction::Seek(new_pos).into())
                .map_err(|_| Error::Failed("Could not send action".to_string()))
        }
    }

    pub fn set_position(&self, TrackId: ObjectPath, Position: i64) -> Result<()> {
        if self.state.current_track().is_none() {
            return Ok(());
        }

        if TrackId.to_string() != self.metadata().id {
            return Ok(());
        }

        let length: i64 = self
            .metadata()
            .length
            .try_into()
            .map_err(|_| Error::Failed("Could not cast length (too large)".to_string()))?;

        if Position > length {
            return Ok(());
        }

        let pos: u32 = (Position / 1000)
            .try_into()
            .map_err(|_| Error::Failed("Could not parse position".to_string()))?;

        self.sender
            .unbounded_send(PlaybackAction::Seek(pos).into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))
    }

    pub fn stop(&self) -> Result<()> {
        Err(Error::NotSupported("Not implemented".to_string()))
    }

    #[dbus_interface(signal)]
    pub async fn seeked(ctxt: &SignalContext<'_>, Position: i64) -> zbus::Result<()>;

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
        self.state
            .current_track()
            .cloned()
            .unwrap_or_else(|| TrackMetadata {
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
    pub fn loop_status(&self) -> LoopStatus {
        self.state.loop_status()
    }

    #[dbus_interface(property)]
    pub fn set_loop_status(&self, value: LoopStatus) -> zbus::Result<()> {
        let mode = match value {
            LoopStatus::None => RepeatMode::None,
            LoopStatus::Track => RepeatMode::Song,
            LoopStatus::Playlist => RepeatMode::Playlist,
        };
        self.sender
            .unbounded_send(PlaybackAction::SetRepeatMode(mode).into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))?;
        Ok(())
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
        self.state.is_shuffled()
    }

    #[dbus_interface(property)]
    pub fn set_shuffle(&self, value: bool) -> zbus::Result<()> {
        self.sender
            .unbounded_send(PlaybackAction::ToggleShuffle.into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))?;
        Ok(())
    }

    #[dbus_interface(property)]
    pub fn volume(&self) -> f64 {
        self.state.volume()
    }

    #[dbus_interface(property)]
    pub fn set_volume(&self, value: f64) -> zbus::Result<()> {
        // As per spec, if new volume less than 0 round to 0
        // also, we don't support volume higher than 100% at the moment.
        let volume = value.clamp(0.0, 1.0);
        self.sender
            .unbounded_send(PlaybackAction::SetVolume(value).into())
            .map_err(|_| Error::Failed("Could not send action".to_string()))?;
        Ok(())
    }
}
