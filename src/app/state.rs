use std::rc::Rc;
use crate::app::{AppAction, AppEvent, AbstractDispatcher};
use crate::app::credentials;
use crate::app::models::*;
use crate::app::browser_state::BrowserState;
use crate::backend::api::SpotifyApiClient;


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

pub struct AppServices {
    pub spotify_api: Rc<dyn SpotifyApiClient>
}

pub struct AppModel {
    pub state: AppState,
    pub dispatcher: Box<dyn AbstractDispatcher<AppAction>>,
    pub services: AppServices
}

impl AppModel {

    pub fn new(
        state: AppState,
        dispatcher: Box<dyn AbstractDispatcher<AppAction>>,
        spotify_api: Rc<dyn SpotifyApiClient>) -> Self {

        let services = AppServices { spotify_api };
        Self { state, dispatcher, services }
    }

    pub fn dispatch(&self, action: AppAction) -> Option<()> {
        self.dispatcher.dispatch(action)
    }

    pub fn update_state(&mut self, message: AppAction) -> Option<AppEvent> {
        match message {
            AppAction::Play => {
                self.state.is_playing = true;
                Some(AppEvent::TrackResumed)
            },
            AppAction::Pause => {
                self.state.is_playing = false;
                Some(AppEvent::TrackPaused)
            },
            AppAction::Next => {
                let next = self.next_song().map(|s| s.uri.clone());
                if next.is_some() {
                    self.state.is_playing = true;
                    self.state.current_song_uri = next.clone();
                    Some(AppEvent::TrackChanged(next.unwrap()))
                } else {
                    None
                }
            },
            AppAction::Previous => {
                let prev = self.prev_song().map(|s| s.uri.clone());
                if prev.is_some() {
                    self.state.is_playing = true;
                    self.state.current_song_uri = prev.clone();
                    Some(AppEvent::TrackChanged(prev.unwrap()))
                } else {
                    None
                }
            },
            AppAction::Load(uri) => {
                self.state.is_playing = true;
                self.state.current_song_uri = Some(uri.clone());
                Some(AppEvent::TrackChanged(uri))
            },
            AppAction::LoadPlaylist(tracks) => {
                self.state.playlist = tracks;
                Some(AppEvent::PlaylistChanged)
            },
            AppAction::LoginSuccess(creds) => {
                let _ = credentials::save_credentials(creds.clone());
                self.services.spotify_api.update_token(&creds.token[..]);
                Some(AppEvent::LoginCompleted)
            },
            AppAction::Seek(pos) => Some(AppEvent::TrackSeeked(pos)),
            AppAction::Start => Some(AppEvent::Started),
            AppAction::TryLogin(u, p) => Some(AppEvent::LoginStarted(u, p)),
            AppAction::BrowserAction(a) => {
                let event = self.state.browser_state.update_with(a)?;
                Some(AppEvent::BrowserEvent(event))
            }
        }
    }


    fn prev_song(&self) -> Option<&SongDescription> {
        let state = &self.state;
        state.current_song_uri.as_ref().and_then(|uri| {
            state.playlist.iter()
                .take_while(|&song| song.uri != *uri)
                .last()
        })
    }

    fn next_song(&self) -> Option<&SongDescription> {
        let state = &self.state;
        state.current_song_uri.as_ref().and_then(|uri| {
            state.playlist.iter()
                .skip_while(|&song| song.uri != *uri)
                .skip(1)
                .next()
        })
    }
}

