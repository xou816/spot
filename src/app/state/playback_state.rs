use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use std::collections::HashMap;

use crate::app::models::SongDescription;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum PlaylistSource {
    Playlist(String),
    Album(String),
    None,
}

impl PartialEq for PlaylistSource {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::Playlist(ref a), &Self::Playlist(ref b)) => a == b,
            (&Self::Album(ref a), &Self::Album(ref b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for PlaylistSource {}

pub struct PlaybackState {
    rng: SmallRng,
    indexed_songs: HashMap<String, SongDescription>,
    running_order: Vec<String>,
    running_order_shuffled: Option<Vec<String>>,
    pub source: PlaylistSource,
    is_playing: bool,
    pub current_song_id: Option<String>,
}

impl PlaybackState {
    pub fn is_playing(&self) -> bool {
        self.is_playing && self.current_song_id.is_some()
    }

    pub fn is_shuffled(&self) -> bool {
        self.running_order_shuffled.is_some()
    }

    pub fn song(&self, id: &str) -> Option<&SongDescription> {
        self.indexed_songs.get(id)
    }

    pub fn songs<'i, 's: 'i>(&'s self) -> impl Iterator<Item = &'i SongDescription> + 'i {
        let iter = self
            .running_order_shuffled
            .as_ref()
            .unwrap_or(&self.running_order);
        let indexed = &self.indexed_songs;
        iter.iter().filter_map(move |id| indexed.get(id))
    }

    pub fn current_song(&self) -> Option<&SongDescription> {
        self.current_song_id
            .as_ref()
            .and_then(|current_song_id| self.song(current_song_id))
    }

    pub fn prev_song(&self) -> Option<&SongDescription> {
        self.current_song_id
            .as_ref()
            .and_then(|id| self.songs().take_while(|&song| song.id != *id).last())
    }

    pub fn next_song(&self) -> Option<&SongDescription> {
        self.current_song_id
            .as_ref()
            .and_then(|id| self.songs().skip_while(|&song| song.id != *id).nth(1))
    }

    fn index_tracks(tracks: Vec<SongDescription>) -> HashMap<String, SongDescription> {
        let mut map: HashMap<String, SongDescription> = HashMap::with_capacity(tracks.len());
        for track in tracks.into_iter() {
            map.insert(track.id.clone(), track);
        }
        map
    }

    fn shuffle(&mut self) {
        let mut to_shuffle: Vec<String> = self
            .running_order
            .iter()
            .filter(|&id| Some(id) != self.current_song_id.as_ref())
            .cloned()
            .collect();
        let mut final_list: Vec<String> = self.current_song_id.iter().cloned().collect();
        to_shuffle.shuffle(&mut self.rng);
        final_list.append(&mut to_shuffle);
        self.running_order_shuffled = Some(final_list);
    }

    fn set_playlist(&mut self, source: PlaylistSource, tracks: Vec<SongDescription>) {
        self.source = source;
        self.running_order = tracks.iter().map(|t| t.id.clone()).collect();
        self.indexed_songs = Self::index_tracks(tracks);
        if self.is_shuffled() {
            self.shuffle();
        }
    }

    fn play(&mut self, id: &str) {
        self.current_song_id = Some(id.to_string());
        self.is_playing = true;
    }

    fn stop(&mut self) {
        self.current_song_id = None;
        self.is_playing = false;
    }

    fn play_next(&mut self) -> Option<String> {
        let id = self.next_song().map(|next| next.id.clone());
        if let Some(id) = id.clone() {
            self.current_song_id = Some(id);
            self.is_playing = true;
        }
        id
    }

    fn play_prev(&mut self) -> Option<String> {
        let id = self.prev_song().map(|prev| prev.id.clone());
        if let Some(id) = id.clone() {
            self.current_song_id = Some(id);
            self.is_playing = true;
        }
        id
    }

    fn toggle_play(&mut self) -> Option<bool> {
        if self.current_song_id.is_some() {
            self.is_playing = !self.is_playing;
            Some(self.is_playing)
        } else {
            None
        }
    }

    fn toggle_shuffle(&mut self) {
        if !self.is_shuffled() {
            self.shuffle();
        } else {
            self.running_order_shuffled = None;
        }
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            indexed_songs: HashMap::new(),
            running_order: vec![],
            running_order_shuffled: None,
            source: PlaylistSource::None,
            is_playing: false,
            current_song_id: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PlaybackAction {
    TogglePlay,
    Play,
    Pause,
    ToggleShuffle,
    Seek(u32),
    SyncSeek(u32),
    Load(String),
    LoadPlaylist(PlaylistSource, Vec<SongDescription>),
    Next,
    Previous,
}

impl Into<AppAction> for PlaybackAction {
    fn into(self) -> AppAction {
        AppAction::PlaybackAction(self)
    }
}

#[derive(Clone, Debug)]
pub enum PlaybackEvent {
    PlaybackPaused,
    PlaybackResumed,
    TrackSeeked(u32),
    SeekSynced(u32),
    TrackChanged(String),
    PlaylistChanged,
    PlaybackStopped,
}

impl Into<AppEvent> for PlaybackEvent {
    fn into(self) -> AppEvent {
        AppEvent::PlaybackEvent(self)
    }
}

impl UpdatableState for PlaybackState {
    type Action = PlaybackAction;
    type Event = PlaybackEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            PlaybackAction::TogglePlay => {
                if let Some(playing) = self.toggle_play() {
                    if playing {
                        vec![PlaybackEvent::PlaybackResumed]
                    } else {
                        vec![PlaybackEvent::PlaybackPaused]
                    }
                } else {
                    vec![]
                }
            }
            PlaybackAction::Play => {
                if !self.is_playing() && self.toggle_play() == Some(true) {
                    vec![PlaybackEvent::PlaybackResumed]
                } else {
                    vec![]
                }
            }
            PlaybackAction::Pause => {
                if self.is_playing() && self.toggle_play() == Some(false) {
                    vec![PlaybackEvent::PlaybackPaused]
                } else {
                    vec![]
                }
            }
            PlaybackAction::ToggleShuffle => {
                self.toggle_shuffle();
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::Next => {
                if let Some(id) = self.play_next() {
                    vec![
                        PlaybackEvent::TrackChanged(id),
                        PlaybackEvent::PlaybackResumed,
                    ]
                } else {
                    self.stop();
                    vec![PlaybackEvent::PlaybackStopped]
                }
            }
            PlaybackAction::Previous => {
                if let Some(id) = self.play_prev() {
                    vec![
                        PlaybackEvent::TrackChanged(id),
                        PlaybackEvent::PlaybackResumed,
                    ]
                } else {
                    vec![]
                }
            }
            PlaybackAction::Load(id) => {
                self.play(&id);
                vec![
                    PlaybackEvent::TrackChanged(id),
                    PlaybackEvent::PlaybackResumed,
                ]
            }
            PlaybackAction::LoadPlaylist(source, tracks) => {
                self.set_playlist(source, tracks);
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::Seek(pos) => vec![PlaybackEvent::TrackSeeked(pos)],
            PlaybackAction::SyncSeek(pos) => vec![PlaybackEvent::SeekSynced(pos)],
        }
    }
}
