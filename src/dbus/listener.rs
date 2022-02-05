use futures::channel::mpsc::UnboundedSender;
use std::rc::Rc;

use crate::app::{
    components::EventListener,
    models::{RepeatMode, SongDescription},
    state::PlaybackEvent,
    AppEvent, AppModel,
};

use super::types::{LoopStatus, PlaybackStatus, TrackMetadata};

#[derive(Debug)]
pub enum MprisStateUpdate {
    SetVolume(f64),
    SetCurrentTrack {
        has_prev: bool,
        current: Option<TrackMetadata>,
        has_next: bool,
    },
    SetPositionMs(u128),
    SetLoopStatus {
        has_prev: bool,
        loop_status: LoopStatus,
        has_next: bool,
    },
    SetShuffled(bool),
    SetPlaying(PlaybackStatus),
}

pub struct AppPlaybackStateListener {
    app_model: Rc<AppModel>,
    sender: UnboundedSender<MprisStateUpdate>,
}

impl AppPlaybackStateListener {
    pub fn new(app_model: Rc<AppModel>, sender: UnboundedSender<MprisStateUpdate>) -> Self {
        Self { app_model, sender }
    }

    fn make_track_meta(&self) -> Option<TrackMetadata> {
        let SongDescription {
            id,
            title,
            artists,
            album,
            duration,
            art,
            ..
        } = self.app_model.get_state().playback.current_song()?;
        Some(TrackMetadata {
            id: format!("/dev/alextren/Spot/Track/{id}"),
            length: 1000 * duration as u64,
            title,
            album: album.name,
            artist: artists.into_iter().map(|a| a.name).collect(),
            art,
        })
    }

    fn has_prev_next(&self) -> (bool, bool) {
        let state = self.app_model.get_state();
        (
            state.playback.prev_index().is_some(),
            state.playback.next_index().is_some(),
        )
    }

    fn is_shuffled(&self) -> bool {
        let state = self.app_model.get_state();
        state.playback.is_shuffled()
    }

    fn loop_status(&self) -> LoopStatus {
        let state = self.app_model.get_state();
        match state.playback.repeat_mode() {
            RepeatMode::None => LoopStatus::None,
            RepeatMode::Song => LoopStatus::Track,
            RepeatMode::Playlist => LoopStatus::Playlist,
        }
    }

    fn update_for(&self, event: &PlaybackEvent) -> Option<MprisStateUpdate> {
        match event {
            PlaybackEvent::PlaybackPaused => {
                Some(MprisStateUpdate::SetPlaying(PlaybackStatus::Paused))
            }
            PlaybackEvent::PlaybackResumed => {
                Some(MprisStateUpdate::SetPlaying(PlaybackStatus::Playing))
            }
            PlaybackEvent::PlaybackStopped => {
                Some(MprisStateUpdate::SetPlaying(PlaybackStatus::Stopped))
            }
            PlaybackEvent::TrackChanged(_) => {
                let current = self.make_track_meta();
                let (has_prev, has_next) = self.has_prev_next();
                Some(MprisStateUpdate::SetCurrentTrack {
                    has_prev,
                    has_next,
                    current,
                })
            }
            PlaybackEvent::RepeatModeChanged(_) => {
                let loop_status = self.loop_status();
                let (has_prev, has_next) = self.has_prev_next();
                Some(MprisStateUpdate::SetLoopStatus {
                    has_prev,
                    has_next,
                    loop_status,
                })
            }
            PlaybackEvent::ShuffleChanged => {
                Some(MprisStateUpdate::SetShuffled(self.is_shuffled()))
            }
            PlaybackEvent::TrackSeeked(pos) | PlaybackEvent::SeekSynced(pos) => {
                let pos = 1000 * (*pos as u128);
                Some(MprisStateUpdate::SetPositionMs(pos))
            }
            PlaybackEvent::VolumeSet(vol) => Some(MprisStateUpdate::SetVolume(*vol)),
            _ => None,
        }
    }
}

impl EventListener for AppPlaybackStateListener {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::PlaybackEvent(event) = event {
            if let Some(update) = self.update_for(event) {
                self.sender
                    .unbounded_send(update)
                    .expect("Could not send event to DBUS server");
            }
        }
    }
}
