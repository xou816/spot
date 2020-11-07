use std::rc::Rc;
use crate::app::state::*;
use crate::app::credentials;
use crate::app::models::*;
use crate::backend::api::SpotifyApiClient;


pub struct AppServices {
    pub spotify_api: Rc<dyn SpotifyApiClient>
}

pub struct AppModel {
    pub state: AppState,
    pub services: AppServices
}

impl AppModel {

    pub fn new(
        state: AppState,
        spotify_api: Rc<dyn SpotifyApiClient>) -> Self {

        let services = AppServices { spotify_api };
        Self { state, services }
    }

    fn event(event: AppEvent) -> Vec<AppEvent> {
        vec![event]
    }

    fn no_event() -> Vec<AppEvent> {
        vec![]
    }

    pub fn update_state(&mut self, message: AppAction) -> Vec<AppEvent> {
        match message {
            AppAction::Play => {
                self.state.is_playing = true;
                Self::event(AppEvent::TrackResumed)
            },
            AppAction::Pause => {
                self.state.is_playing = false;
                Self::event(AppEvent::TrackPaused)
            },
            AppAction::Next => {
                let next = self.next_song().map(|s| s.uri.clone());
                if next.is_some() {
                    self.state.is_playing = true;
                    self.state.current_song_uri = next.clone();
                    Self::event(AppEvent::TrackChanged(next.unwrap()))
                } else {
                    Self::no_event()
                }
            },
            AppAction::Previous => {
                let prev = self.prev_song().map(|s| s.uri.clone());
                if prev.is_some() {
                    self.state.is_playing = true;
                    self.state.current_song_uri = prev.clone();
                    Self::event(AppEvent::TrackChanged(prev.unwrap()))
                } else {
                    Self::no_event()
                }
            },
            AppAction::Load(uri) => {
                self.state.is_playing = true;
                self.state.current_song_uri = Some(uri.clone());
                Self::event(AppEvent::TrackChanged(uri))
            },
            AppAction::LoadPlaylist(tracks) => {
                self.state.playlist = tracks;
                Self::event(AppEvent::PlaylistChanged)
            },
            AppAction::LoginSuccess(creds) => {
                let _ = credentials::save_credentials(creds.clone());
                self.services.spotify_api.update_token(&creds.token[..]);
                Self::event(AppEvent::LoginCompleted)
            },
            AppAction::Seek(pos) => Self::event(AppEvent::TrackSeeked(pos)),
            AppAction::Start => Self::event(AppEvent::Started),
            AppAction::TryLogin(u, p) => Self::event(AppEvent::LoginStarted(u, p)),
            AppAction::BrowserAction(a) => self.state
                .browser_state
                .update_with(a)
                .into_iter()
                .map(|e| AppEvent::BrowserEvent(e))
                .collect()
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

