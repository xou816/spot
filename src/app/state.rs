use super::AppAction;
use super::components::{PlaybackState, PlaylistState};


#[derive(Clone)]
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

impl PlaybackState for AppState {
    fn is_playing(&self) -> bool {
        self.is_playing && self.current_song_uri.is_some()
    }

    fn current_song(&self) -> Option<&SongDescription> {
        self.current_song_uri.as_ref().and_then(|uri| {
            self.playlist.iter().find(|&song| song.uri == *uri)
        })
    }

    fn next_song_action(&self) -> Option<AppAction> {
        let next_song = self.current_song_uri.as_ref().and_then(|uri| {
            self.playlist.iter()
                .skip_while(|&song| song.uri != *uri)
                .skip(1)
                .next()
        });
        next_song.map(|song| AppAction::Load(song.uri.clone()))
    }

    fn prev_song_action(&self) -> Option<AppAction> {
        let prev_song = self.current_song_uri.as_ref().and_then(|uri| {
            self.playlist.iter()
                .take_while(|&song| song.uri != *uri)
                .last()
        });
        prev_song.map(|song| AppAction::Load(song.uri.clone()))
    }
}

impl PlaylistState for AppState {

    fn current_song_uri(&self) -> Option<String> {
        self.current_song_uri.clone()
    }

    fn songs(&self) -> &Vec<SongDescription> {
        &self.playlist
    }
}
