use std::rc::Rc;
use std::cell::RefCell;

use super::AppAction;
use super::components::{PlaybackModel, PlaylistModel, LoginModel, BrowserModel};
use super::backend::api::SpotifyApi;
use super::{Dispatcher, Worker};
use super::credentials;


#[derive(Clone, Debug)]
pub struct AlbumDescription {
    pub title: String,
    pub artist: String,
    pub uri: String,
    pub art: String,
    pub songs: Vec<SongDescription>,
    pub id: String
}

#[derive(Clone, Debug)]
pub struct SongDescription {
    pub title: String,
    pub artist: String,
    pub uri: String
}

impl SongDescription {
    pub fn new(title: &str, artist: &str, uri: &str) -> Self {
        Self { title: title.to_string(), artist: artist.to_string(), uri: uri.to_string() }
    }
}

pub struct AppModel {
    pub state: AppState,
    pub dispatcher: Dispatcher,
    pub worker: Worker,
    pub api: Rc<RefCell<SpotifyApi>>
}

impl AppModel {
    pub fn new(state: AppState, dispatcher: Dispatcher, worker: Worker) -> Self {
        Self {
            state, dispatcher, worker,
            api: Rc::new(RefCell::new(SpotifyApi::new()))
        }
    }
}

pub struct AppState {
    pub is_playing: bool,
    pub current_song_uri: Option<String>,
    pub playlist: Vec<SongDescription>
}

impl AppState {
    pub fn new(songs: Vec<SongDescription>) -> Self {
        Self {
            is_playing: false,
            current_song_uri: None,
            playlist: songs
        }
    }
}


impl PlaybackModel for AppModel {
    fn is_playing(&self) -> bool {
        self.state.is_playing && self.state.current_song_uri.is_some()
    }

    fn current_song(&self) -> Option<&SongDescription> {
        self.state.current_song_uri.as_ref().and_then(|uri| {
            self.state.playlist.iter().find(|&song| song.uri == *uri)
        })
    }

    fn play_next_song(&self) {
        let next_song = self.state.current_song_uri.as_ref().and_then(|uri| {
            self.state.playlist.iter()
                .skip_while(|&song| song.uri != *uri)
                .skip(1)
                .next()
        });
        let action = next_song.map(|song| AppAction::Load(song.uri.clone()));
        self.dispatcher.dispatch(action);
    }

    fn play_prev_song(&self) {
        let prev_song = self.state.current_song_uri.as_ref().and_then(|uri| {
            self.state.playlist.iter()
                .take_while(|&song| song.uri != *uri)
                .last()
        });
        let action = prev_song.map(|song| AppAction::Load(song.uri.clone()));
        self.dispatcher.dispatch(action);
    }

    fn toggle_playback(&self) {
        self.dispatcher.dispatch(if self.is_playing() {
            AppAction::Pause
        } else {
            AppAction::Play
        });
    }
}

impl PlaylistModel for AppModel {

    fn current_song_uri(&self) -> Option<String> {
        self.state.current_song_uri.clone()
    }

    fn songs(&self) -> &Vec<SongDescription> {
        &self.state.playlist
    }

    fn play_song(&self, uri: String) {
        self.dispatcher.dispatch(AppAction::Load(uri));
    }
}

impl LoginModel for AppModel {

    fn try_autologin(&self) -> bool {
        if let Ok(creds) = credentials::try_retrieve_credentials() {
            self.dispatcher.dispatch(AppAction::TryLogin(creds.username, creds.password));
            true
        } else {
            false
        }
    }

    fn login(&self, u: String, p: String) {
        self.dispatcher.dispatch(AppAction::TryLogin(u, p));
    }
}

impl BrowserModel for AppModel {

    fn get_saved_albums(&self, completion: Box<dyn Fn(Vec<AlbumDescription>) -> ()>) {
        let api = Rc::clone(&self.api);
        self.worker.send_task(async move {
            if let Some(albums) = api.borrow().get_saved_albums().await {
                completion(albums);
            }
        });
    }

    fn play_album(&self, album_uri: &str) {
        let api = Rc::clone(&self.api);
        let dispatcher = self.dispatcher.clone();
        let uri = String::from(album_uri);
        self.worker.send_task(async move {
            if let Some(songs) = api.borrow().get_album(&uri).await {
                let first = songs[0].uri.clone();
                dispatcher.dispatch(AppAction::LoadPlaylist(songs));
                dispatcher.dispatch(AppAction::Load(first));
            }
        });
    }
}
