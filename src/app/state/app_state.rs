use crate::app::credentials;
use crate::app::models::*;
use crate::app::state::{BrowserState, BrowserAction, BrowserEvent, UpdatableState};

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

    pub fn update_state(&mut self, message: AppAction) -> Vec<AppEvent> {
        match message {
            AppAction::Play => {
                self.is_playing = true;
                vec![AppEvent::TrackResumed]
            },
            AppAction::Pause => {
                self.is_playing = false;
                vec![AppEvent::TrackPaused]
            },
            AppAction::Next => {
                let next = self.next_song().map(|s| s.uri.clone());
                if next.is_some() {
                    self.is_playing = true;
                    self.current_song_uri = next.clone();
                    vec![AppEvent::TrackChanged(next.unwrap())]
                } else {
                    vec![]
                }
            },
            AppAction::Previous => {
                let prev = self.prev_song().map(|s| s.uri.clone());
                if prev.is_some() {
                    self.is_playing = true;
                    self.current_song_uri = prev.clone();
                    vec![AppEvent::TrackChanged(prev.unwrap())]
                } else {
                    vec![]
                }
            },
            AppAction::Load(uri) => {
                self.is_playing = true;
                self.current_song_uri = Some(uri.clone());
                vec![AppEvent::TrackChanged(uri)]
            },
            AppAction::LoadPlaylist(tracks) => {
                self.playlist = tracks;
                vec![AppEvent::PlaylistChanged]
            },
            AppAction::LoginSuccess(_) => vec![AppEvent::LoginCompleted],
            AppAction::Seek(pos) => vec![AppEvent::TrackSeeked(pos)],
            AppAction::SyncSeek(pos) => vec![AppEvent::SeekSynced(pos)],
            AppAction::Start => vec![AppEvent::Started],
            AppAction::TryLogin(u, p) => vec![AppEvent::LoginStarted(u, p)],
            AppAction::BrowserAction(a) => self.browser_state
                .update_with(a)
                .into_iter()
                .map(|e| AppEvent::BrowserEvent(e))
                .collect()
        }
    }

    fn prev_song(&self) -> Option<&SongDescription> {
        self.current_song_uri.as_ref().and_then(|uri| {
            self.playlist.iter()
                .take_while(|&song| song.uri != *uri)
                .last()
        })

    }

    fn next_song(&self) -> Option<&SongDescription> {
        self.current_song_uri.as_ref().and_then(|uri| {
            self.playlist.iter()
                .skip_while(|&song| song.uri != *uri)
                .skip(1)
                .next()
        })
    }
}

