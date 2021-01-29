use std::convert::Into;
use std::sync::{Arc, Mutex};
use zvariant::Type;
use zvariant::{Dict, Signature, Str, Value};

fn boxed_value<'a, V: Into<Value<'a>>>(v: V) -> Value<'a> {
    Value::new(v.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
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
            PlaybackStatus::Stopped => "Stopped".into(),
        }
    }
}

#[derive(Clone)]
pub struct TrackMetadata {
    pub artist: Vec<String>,
    pub title: String,
}

impl Type for TrackMetadata {
    fn signature() -> Signature<'static> {
        Signature::from_str_unchecked("a{sv}")
    }
}

impl From<TrackMetadata> for Value<'_> {
    fn from(meta: TrackMetadata) -> Self {
        let mut d = Dict::new(Str::signature(), Value::signature());
        d.append("xesam:title".into(), boxed_value(meta.title))
            .unwrap();
        d.append("xesam:artist".into(), boxed_value(meta.artist.clone()))
            .unwrap();
        d.append("xesam:albumArtist".into(), boxed_value(meta.artist))
            .unwrap();
        Value::Dict(d)
    }
}

struct MprisState {
    status: PlaybackStatus,
    metadata: Option<TrackMetadata>,
}

#[derive(Clone)]
pub struct SharedMprisState(Arc<Mutex<MprisState>>);

impl SharedMprisState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(MprisState {
            status: PlaybackStatus::Stopped,
            metadata: None,
        })))
    }

    pub fn status(&self) -> PlaybackStatus {
        self.0
            .lock()
            .map(|s| s.status)
            .unwrap_or(PlaybackStatus::Stopped)
    }

    pub fn current_track(&self) -> Option<TrackMetadata> {
        self.0.lock().ok().and_then(|s| s.metadata.clone()) // clone :(
    }

    pub fn set_playing(&self, playing: bool) {
        if let Ok(mut state) = self.0.lock() {
            (*state).status = if playing {
                PlaybackStatus::Playing
            } else {
                PlaybackStatus::Paused
            }
        }
    }

    pub fn toggle_playing(&self) {
        if let Ok(mut state) = self.0.lock() {
            let current = state.status;
            (*state).status = match current {
                PlaybackStatus::Playing => PlaybackStatus::Paused,
                _ => PlaybackStatus::Playing,
            }
        }
    }
}
