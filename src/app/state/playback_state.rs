use rand::{rngs::SmallRng, seq::SliceRandom, RngCore, SeedableRng};
use std::collections::HashMap;

use super::pagination::Pagination;
use crate::app::models::SongDescription;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum PlaylistSource {
    Playlist(String),
    Album(String),
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
    pub pagination: Option<Pagination<PlaylistSource>>,
    is_playing: bool,
    pub current_song_id: Option<String>,
}

const QUEUE_SIZE: u32 = 100;

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

    pub fn songs<'s>(&'s self) -> impl Iterator<Item = &'s SongDescription> + 's {
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

    fn set_playlist(&mut self, source: Option<PlaylistSource>, tracks: Vec<SongDescription>) {
        self.pagination = source.map(|source| {
            let mut p = Pagination::new(source, QUEUE_SIZE);
            p.reset_count(tracks.len() as u32);
            p
        });
        self.running_order = tracks.iter().map(|t| t.id.clone()).collect();
        self.indexed_songs = Self::index_tracks(tracks);
        if self.is_shuffled() {
            self.shuffle();
        }
    }

    pub fn queue(&mut self, track: SongDescription) {
        if !self.indexed_songs.contains_key(&track.id) {
            self.pagination = None;
            self.running_order.push(track.id.clone());
            if let Some(shuffled) = self.running_order_shuffled.as_mut() {
                let next = (self.rng.next_u32() as usize) % (shuffled.len() - 1);
                shuffled.insert(next + 1, track.id.clone());
            }
            self.indexed_songs.insert(track.id.clone(), track);
        }
    }

    pub fn dequeue(&mut self, id: &str) {
        if self.indexed_songs.contains_key(id) {
            self.running_order.retain(|t| t != id);
            if let Some(shuffled) = self.running_order_shuffled.as_mut() {
                shuffled.retain(|t| t != id);
            }
            self.indexed_songs.remove(id);
        }
    }

    pub fn move_down(&mut self, id: &str) -> bool {
        let len = self.running_order.len();
        let running_order = self
            .running_order_shuffled
            .as_mut()
            .unwrap_or(&mut self.running_order);
        let index = running_order
            .iter()
            .position(|s| s == id)
            .filter(|&index| index + 1 < len);
        if let Some(index) = index {
            running_order.swap(index, index + 1);
            true
        } else {
            false
        }
    }

    pub fn move_up(&mut self, id: &str) -> bool {
        let running_order = self
            .running_order_shuffled
            .as_mut()
            .unwrap_or(&mut self.running_order);
        let prev_index = running_order
            .iter()
            .position(|s| s == id)
            .filter(|&index| index > 0)
            .map(|index| index.saturating_sub(1));
        if let Some(prev_index) = prev_index {
            running_order.swap(prev_index, prev_index + 1);
            true
        } else {
            false
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

    pub fn source(&self) -> Option<&PlaylistSource> {
        self.pagination.as_ref().map(|p| &p.data)
    }

    pub fn position(&self) -> usize {
        match self.current_song_id.as_ref() {
            Some(id) => self.songs().position(|s| id == &s.id).unwrap_or(0),
            None => 0,
        }
    }

    pub fn max_size(&self) -> usize {
        QUEUE_SIZE as usize
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            indexed_songs: HashMap::new(),
            running_order: vec![],
            running_order_shuffled: None,
            pagination: None,
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
    Stop,
    ToggleShuffle,
    Seek(u32),
    SyncSeek(u32),
    Load(String),
    LoadPlaylist(Option<PlaylistSource>, Vec<SongDescription>),
    Next,
    Previous,
    Queue(SongDescription),
    QueueMany(Vec<SongDescription>),
    Dequeue(String),
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
            PlaybackAction::Stop => {
                self.stop();
                vec![PlaybackEvent::PlaybackStopped]
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
                if self.current_song_id.as_ref() != Some(&id) {
                    self.play(&id);
                    vec![
                        PlaybackEvent::TrackChanged(id),
                        PlaybackEvent::PlaybackResumed,
                    ]
                } else {
                    vec![]
                }
            }
            PlaybackAction::LoadPlaylist(source, tracks) => {
                self.set_playlist(source, tracks);
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::Queue(track) => {
                self.queue(track);
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::QueueMany(tracks) => {
                for track in tracks {
                    self.queue(track);
                }
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::Dequeue(id) => {
                self.dequeue(&id);
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::Seek(pos) => vec![PlaybackEvent::TrackSeeked(pos)],
            PlaybackAction::SyncSeek(pos) => vec![PlaybackEvent::SeekSynced(pos)],
        }
    }
}
