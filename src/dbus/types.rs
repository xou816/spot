use std::convert::{Into, TryFrom};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use zvariant::Type;
use zvariant::{Dict, Signature, Str, Value};

fn boxed_value<'a, V: Into<Value<'a>>>(v: V) -> Value<'a> {
    Value::new(v.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopStatus {
    None,
    Track,
    Playlist,
}

impl Type for LoopStatus {
    fn signature() -> Signature<'static> {
        Str::signature()
    }
}

impl TryFrom<&Value<'_>> for LoopStatus {
    type Error = zvariant::Error;

    fn try_from(value: &Value<'_>) -> Result<Self, Self::Error> {
        let s = String::try_from(value)?;
        let s = s.as_str();
        Ok(match s {
            "Track" => LoopStatus::Track,
            "Playlist" => LoopStatus::Playlist,
            _ => LoopStatus::None,
        })
    }
}

impl From<LoopStatus> for Value<'_> {
    fn from(status: LoopStatus) -> Self {
        match status {
            LoopStatus::None => "None".into(),
            LoopStatus::Track => "Track".into(),
            LoopStatus::Playlist => "Playlist".into(),
        }
    }
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

struct PositionMicros {
    last_known_position: u128,
    last_resume_instant: Option<Instant>,
    rate: f32,
}

impl PositionMicros {
    fn new(rate: f32) -> Self {
        Self {
            last_known_position: 0,
            last_resume_instant: None,
            rate,
        }
    }
    fn current(&self) -> u128 {
        let current_progress = self.last_resume_instant.map(|ri| {
            let elapsed = ri.elapsed().as_micros() as f32;
            let real_elapsed = self.rate * elapsed;
            real_elapsed.ceil() as u128
        });
        self.last_known_position + current_progress.unwrap_or(0)
    }

    fn set(&mut self, position: u128, playing: bool) {
        self.last_known_position = position;
        self.last_resume_instant = if playing { Some(Instant::now()) } else { None }
    }

    fn pause(&mut self) {
        self.last_known_position = self.current();
        self.last_resume_instant = None;
    }

    fn resume(&mut self) {
        self.last_resume_instant = Some(Instant::now());
    }
}

#[derive(Clone)]
pub struct TrackMetadata {
    pub id: String,
    pub length: u64,
    pub artist: Vec<String>,
    pub album: String,
    pub title: String,
    pub art: Option<String>,
}

impl Type for TrackMetadata {
    fn signature() -> Signature<'static> {
        Signature::from_str_unchecked("a{sv}")
    }
}

impl From<TrackMetadata> for Value<'_> {
    fn from(meta: TrackMetadata) -> Self {
        let mut d = Dict::new(Str::signature(), Value::signature());
        d.append("mpris:trackid".into(), boxed_value(meta.id))
            .unwrap();
        d.append("mpris:length".into(), boxed_value(meta.length))
            .unwrap();
        d.append("xesam:title".into(), boxed_value(meta.title))
            .unwrap();
        d.append("xesam:artist".into(), boxed_value(meta.artist.clone()))
            .unwrap();
        d.append("xesam:albumArtist".into(), boxed_value(meta.artist))
            .unwrap();
        d.append("xesam:album".into(), boxed_value(meta.album))
            .unwrap();
        if let Some(art) = meta.art {
            d.append("mpris:artUrl".into(), boxed_value(art)).unwrap();
        }
        Value::Dict(d)
    }
}

struct MprisState {
    status: PlaybackStatus,
    loop_status: LoopStatus,
    volume: f64,
    shuffled: bool,
    position: PositionMicros,
    metadata: Option<TrackMetadata>,
    has_prev: bool,
    has_next: bool,
}

#[derive(Clone)]
pub struct SharedMprisState(Arc<Mutex<MprisState>>);

impl SharedMprisState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(MprisState {
            status: PlaybackStatus::Stopped,
            loop_status: LoopStatus::None,
            shuffled: false,
            position: PositionMicros::new(1.0),
            metadata: None,
            has_prev: false,
            has_next: false,
            volume: 1f64,
        })))
    }

    pub fn volume(&self) -> f64 {
        self.0
            .lock()
            .map(|s| s.volume)
            .map_err(|e| error!("Failed to get volume: {:?}", e))
            .unwrap_or(1.0f64)
    }

    pub fn set_volume(&self, volume: f64) {
        if let Ok(mut state) = self.0.lock() {
            (*state).volume = volume;
        }
    }

    pub fn status(&self) -> PlaybackStatus {
        self.0
            .lock()
            .map(|s| s.status)
            .unwrap_or(PlaybackStatus::Stopped)
    }

    pub fn loop_status(&self) -> LoopStatus {
        self.0
            .lock()
            .map(|s| s.loop_status)
            .unwrap_or(LoopStatus::None)
    }

    pub fn is_shuffled(&self) -> bool {
        self.0.lock().ok().map(|s| s.shuffled).unwrap_or(false)
    }

    pub fn current_track(&self) -> Option<TrackMetadata> {
        self.0.lock().ok().and_then(|s| s.metadata.clone())
    }

    pub fn has_prev(&self) -> bool {
        self.0.lock().ok().map(|s| s.has_prev).unwrap_or(false)
    }

    pub fn has_next(&self) -> bool {
        self.0.lock().ok().map(|s| s.has_next).unwrap_or(false)
    }

    pub fn set_has_prev(&self, has_prev: bool) {
        if let Ok(mut state) = self.0.lock() {
            (*state).has_prev = has_prev;
        }
    }

    pub fn set_has_next(&self, has_next: bool) {
        if let Ok(mut state) = self.0.lock() {
            (*state).has_next = has_next;
        }
    }

    pub fn set_current_track(&self, track: Option<TrackMetadata>) {
        if let Ok(mut state) = self.0.lock() {
            let playing = state.status == PlaybackStatus::Playing;
            (*state).metadata = track;
            (*state).position.set(0, playing);
        }
    }

    pub fn position(&self) -> u128 {
        self.0
            .lock()
            .ok()
            .map(|s| s.position.current())
            .unwrap_or(0)
    }

    pub fn set_position(&self, position: u128) {
        if let Ok(mut state) = self.0.lock() {
            let playing = state.status == PlaybackStatus::Playing;
            (*state).position.set(position, playing);
        }
    }

    pub fn set_loop_status(&self, loop_status: LoopStatus) {
        if let Ok(mut state) = self.0.lock() {
            (*state).loop_status = loop_status;
        }
    }

    pub fn set_shuffled(&self, shuffled: bool) {
        if let Ok(mut state) = self.0.lock() {
            (*state).shuffled = shuffled;
        }
    }

    pub fn set_playing(&self, status: PlaybackStatus) {
        if let Ok(mut state) = self.0.lock() {
            (*state).status = status;
            match status {
                PlaybackStatus::Playing => {
                    (*state).position.resume();
                }
                PlaybackStatus::Paused => {
                    (*state).position.pause();
                }
                PlaybackStatus::Stopped => {
                    (*state).position.set(0, false);
                }
            }
        }
    }
}
