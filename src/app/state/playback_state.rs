use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use std::collections::HashMap;

use crate::app::models::SongDescription;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

pub struct PlayQueue {
    rng: SmallRng,
    indexed_songs: HashMap<String, SongDescription>,
    running_order: Vec<String>,
    running_order_shuffled: Option<Vec<String>>,
}

impl PlayQueue {
    pub fn new(tracks: &[SongDescription]) -> Self {
        let running_order = tracks.iter().map(|t| t.id.clone()).collect();
        PlayQueue {
            rng: SmallRng::from_entropy(),
            indexed_songs: Self::index_tracks(tracks),
            running_order,
            running_order_shuffled: None,
        }
    }

    fn index_tracks(tracks: &[SongDescription]) -> HashMap<String, SongDescription> {
        let mut map: HashMap<String, SongDescription> = HashMap::with_capacity(tracks.len());
        for track in tracks.iter() {
            map.insert(track.id.clone(), track.clone());
        }
        map
    }

    fn is_shuffled(&self) -> bool {
        self.running_order_shuffled.is_some()
    }

    pub fn set_tracks(&mut self, tracks: &[SongDescription], keep_index: Option<usize>) {
        self.indexed_songs = Self::index_tracks(tracks);
        if self.is_shuffled() {
            self.shuffle(keep_index);
        }
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

    pub fn shuffle(&mut self, keep_index: Option<usize>) {
        let mut shuffled = self.running_order.clone();
        let mut final_list = if let Some(index) = keep_index {
            vec![shuffled.remove(index)]
        } else {
            vec![]
        };
        shuffled.shuffle(&mut self.rng);
        final_list.append(&mut shuffled);
        self.running_order_shuffled = Some(final_list);
    }

    pub fn toggle_shuffle(&mut self, keep_index: Option<usize>) {
        if !self.is_shuffled() {
            self.shuffle(keep_index);
        }
    }
}

#[derive(Clone, Debug)]
pub enum PlaybackAction {
    TogglePlay,
    ToggleShuffle,
    Seek(u32),
    SyncSeek(u32),
    Load(String),
    LoadPlaylist(Vec<SongDescription>),
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
    TrackPaused,
    TrackResumed,
    TrackSeeked(u32),
    SeekSynced(u32),
    TrackChanged(String),
    PlaylistChanged,
}

impl Into<AppEvent> for PlaybackEvent {
    fn into(self) -> AppEvent {
        AppEvent::PlaybackEvent(self)
    }
}


pub struct PlaybackState {
    pub is_playing: bool,
    pub current_song_id: Option<String>,
    pub playlist: PlayQueue,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            current_song_id: None,
            playlist: PlayQueue::new(&[]),
        }
    }
}

impl PlaybackState {
    pub fn current_song(&self) -> Option<SongDescription> {
        self.current_song_id
            .as_ref()
            .and_then(|current_song_id| self.playlist.song(current_song_id).cloned())
    }

    pub fn prev_song(&self) -> Option<&SongDescription> {
        self.current_song_id.as_ref().and_then(|id| {
            self.playlist
                .songs()
                .take_while(|&song| song.id != *id)
                .last()
        })
    }

    pub fn next_song(&self) -> Option<&SongDescription> {
        self.current_song_id.as_ref().and_then(|id| {
            self.playlist
                .songs()
                .skip_while(|&song| song.id != *id)
                .nth(1)
        })
    }
}

impl UpdatableState for PlaybackState {
    type Action = PlaybackAction;
    type Event = PlaybackEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            PlaybackAction::TogglePlay => {
                if self.is_playing {
                    self.is_playing = false;
                    vec![PlaybackEvent::TrackPaused]
                } else if self.current_song_id.is_some() {
                    self.is_playing = true;
                    vec![PlaybackEvent::TrackResumed]
                } else {
                    vec![]
                }
            }
            PlaybackAction::ToggleShuffle => {
                //self.playlist.toggle_shuffle(self.song_index());
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::Next => {
                let next = self.next_song().map(|s| s.id.clone());
                if next.is_some() {
                    self.is_playing = true;
                    self.current_song_id = next.clone();
                    vec![
                        PlaybackEvent::TrackChanged(next.unwrap()),
                        PlaybackEvent::TrackResumed,
                    ]
                } else {
                    vec![]
                }
            }
            PlaybackAction::Previous => {
                let prev = self.prev_song().map(|s| s.id.clone());
                if prev.is_some() {
                    self.is_playing = true;
                    self.current_song_id = prev.clone();
                    vec![
                        PlaybackEvent::TrackChanged(prev.unwrap()),
                        PlaybackEvent::TrackResumed,
                    ]
                } else {
                    vec![]
                }
            }
            PlaybackAction::Load(id) => {
                self.is_playing = true;
                self.current_song_id = Some(id.clone());
                vec![PlaybackEvent::TrackChanged(id), PlaybackEvent::TrackResumed]
            }
            PlaybackAction::LoadPlaylist(tracks) => {
                self.playlist.set_tracks(&tracks, None);
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::Seek(pos) => vec![PlaybackEvent::TrackSeeked(pos)],
            PlaybackAction::SyncSeek(pos) => vec![PlaybackEvent::SeekSynced(pos)],
        }
    }
}
