use crate::app::credentials;
use crate::app::models::*;
use crate::app::state::{BrowserState, BrowserAction, BrowserEvent};

#[derive(Clone, Debug)]
pub enum AppAction {
    Play,
    Pause,
    Seek(u32),
    SyncSeek(u32),
    Load(String),
    LoadPlaylist(Vec<SongDescription>),
    Start,
    TryLogin(String, String),
    LoginSuccess(credentials::Credentials),
    Next,
    Previous,
    BrowserAction(BrowserAction)
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Started,
    TrackPaused,
    TrackResumed,
    TrackSeeked(u32),
    SeekSynced(u32),
    LoginStarted(String, String),
    LoginCompleted,
    TrackChanged(String),
    PlaylistChanged,
    BrowserEvent(BrowserEvent)
}

pub struct AppState {
    pub is_playing: bool,
    pub current_song_uri: Option<String>,
    pub playlist: Vec<SongDescription>,
    pub browser_state: BrowserState
}

impl AppState {
    pub fn new(songs: Vec<SongDescription>) -> Self {
        Self {
            is_playing: false,
            current_song_uri: None,
            playlist: songs,
            browser_state: BrowserState::new()
        }
    }
}

