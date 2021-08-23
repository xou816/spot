use rand::{rngs::SmallRng, seq::SliceRandom, RngCore, SeedableRng};
use std::collections::{HashMap, VecDeque};

use crate::app::models::{Batch, SongBatch, SongDescription};
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

const RANGE_SIZE: usize = 25;
pub const QUEUE_DEFAULT_SIZE: usize = 100;

#[derive(Debug, Copy, Clone)]
struct Position {
    pub index: usize,
    pub start: usize,
    pub count: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            index: 0,
            start: 0,
            count: 2 * RANGE_SIZE,
        }
    }
}

impl Position {
    fn has_range_moved(old: Option<Self>, new: Option<Self>) -> bool {
        old.and_then(|old| new.map(|new| old.start != new.start))
            .unwrap_or(true)
    }

    fn update_into(self, pos: usize, max: usize) -> Self {
        let cutoff = RANGE_SIZE.saturating_sub(max - pos);
        let start = pos.saturating_sub(RANGE_SIZE + cutoff);
        let count = usize::min(2 * RANGE_SIZE, max - start);
        Self {
            index: usize::min(pos, max - 1),
            start,
            count,
        }
    }

    fn update(&mut self, pos: usize, max: usize) {
        let s = *self;
        *self = s.update_into(pos, max);
    }

    fn update_count(&mut self, max: usize) {
        let s = *self;
        *self = s.update_into(self.index, max);
    }

    fn decrement(&mut self) {
        let s = *self;
        *self = s.update_into(self.index.saturating_sub(1), self.count.saturating_sub(1));
    }
}

pub struct PlaybackState {
    rng: SmallRng,
    indexed_songs: HashMap<String, SongDescription>,
    running_order: VecDeque<String>,
    running_order_shuffled: Option<VecDeque<String>>,
    position: Option<Position>,
    pub source: Option<PlaylistSource>,
    current_batch: Option<Batch>,
    repeat: RepeatMode,
    is_playing: bool,
}

impl PlaybackState {
    pub fn current_offset(&self) -> Option<usize> {
        self.position.map(|p| p.start)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing && self.position.is_some()
    }

    pub fn is_shuffled(&self) -> bool {
        self.running_order_shuffled.is_some()
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat
    }

    pub fn next_batch(&self) -> Option<Batch> {
        self.current_batch?.next()
    }

    pub fn song(&self, id: &str) -> Option<&SongDescription> {
        self.indexed_songs.get(id)
    }

    fn running_order(&self) -> &VecDeque<String> {
        self.running_order_shuffled
            .as_ref()
            .unwrap_or(&self.running_order)
    }

    fn running_order_mut(&mut self) -> &mut VecDeque<String> {
        self.running_order_shuffled
            .as_mut()
            .unwrap_or(&mut self.running_order)
    }

    pub fn songs(&self) -> impl Iterator<Item = &'_ SongDescription> + '_ {
        let indexed = &self.indexed_songs;
        let Position { start, count, .. } = self.position.unwrap_or_default();
        self.running_order()
            .iter()
            //.skip(start)
            //.take(count)
            .filter_map(move |id| indexed.get(id))
    }

    pub fn current_song_id(&self) -> Option<&String> {
        self.position.map(|pos| &self.running_order()[pos.index])
    }

    pub fn current_song(&self) -> Option<&SongDescription> {
        self.current_song_id().and_then(|id| self.song(id))
    }

    pub fn prev_song(&self) -> Option<&SongDescription> {
        self.prev_index()
            .map(|i| &self.running_order()[i])
            .and_then(|id| self.song(id))
    }

    pub fn next_song(&self) -> Option<&SongDescription> {
        self.next_index()
            .map(|i| &self.running_order()[i])
            .and_then(|id| self.song(id))
    }

    fn index_tracks(
        tracks: impl Iterator<Item = SongDescription>,
    ) -> HashMap<String, SongDescription> {
        let mut map: HashMap<String, SongDescription> =
            HashMap::with_capacity(tracks.size_hint().1.unwrap_or(QUEUE_DEFAULT_SIZE));
        for track in tracks {
            map.insert(track.id.clone(), track);
        }
        map
    }

    fn shuffle(&mut self) {
        let current_song = self.current_song_id();
        let mut to_shuffle: Vec<String> = self
            .running_order
            .iter()
            .filter(|&id| Some(id) != current_song)
            .cloned()
            .collect();
        let mut final_list: VecDeque<String> = current_song.cloned().into_iter().collect();
        to_shuffle.shuffle(&mut self.rng);
        final_list.append(&mut to_shuffle.into());
        if let Some(p) = self.position.as_mut() {
            p.update(0, final_list.len())
        }
        self.running_order_shuffled = Some(final_list);
    }

    fn set_playlist(
        &mut self,
        source: Option<PlaylistSource>,
        current_batch: Option<Batch>,
        tracks: Vec<SongDescription>,
    ) {
        self.position = None;
        self.current_batch = current_batch;
        self.source = source;
        self.running_order = tracks.iter().map(|t| t.id.clone()).collect();
        self.indexed_songs = Self::index_tracks(tracks.into_iter());
        if self.is_shuffled() {
            self.shuffle();
        }
    }

    pub fn queue(&mut self, track: SongDescription) {
        if self.indexed_songs.contains_key(&track.id) {
            return;
        }

        self.running_order.push_back(track.id.clone());
        if let Some(shuffled) = self.running_order_shuffled.as_mut() {
            let next = (self.rng.next_u32() as usize) % (shuffled.len() - 1);
            shuffled.insert(next + 1, track.id.clone());
        }

        self.indexed_songs.insert(track.id.clone(), track);

        let max = self.running_order().len();
        if let Some(position) = self.position.as_mut() {
            position.update_count(max);
        }
    }

    pub fn dequeue(&mut self, id: &str) {
        if !self.indexed_songs.contains_key(id) {
            return;
        }

        if let Some(position) = self.running_order().iter().position(|t| t == id) {
            let new_max = {
                let running_order = self.running_order_mut();
                running_order.remove(position);
                running_order.len()
            };
            let current_comes_after = self.position.map(|p| p.index >= position).unwrap_or(false);
            // fix the position of the current song
            if current_comes_after {
                if new_max > 0 {
                    if let Some(position) = self.position.as_mut() {
                        position.decrement();
                    }
                } else {
                    self.position = None;
                }
            }
        }

        // if the playlist is shuffled, we also need to remove the track from the unshuffled list
        if self.is_shuffled() {
            self.running_order.retain(|t| t != id);
        }

        self.indexed_songs.remove(id);
    }

    fn swap(&mut self, index: usize, other_index: usize) {
        let max = self.running_order().len();
        let running_order = self.running_order_mut();
        running_order.swap(index, other_index);
        if let Some(position) = self.position.as_mut() {
            if index == position.index {
                position.update(other_index, max);
            } else if other_index == position.index {
                position.update(index, max);
            }
        }
    }

    pub fn move_down(&mut self, id: &str) -> bool {
        let running_order = self.running_order();
        let len = running_order.len();
        let index = running_order
            .iter()
            .position(|s| s == id)
            .filter(|&index| index + 1 < len);
        if let Some(index) = index {
            self.swap(index, index + 1);
            true
        } else {
            false
        }
    }

    pub fn move_up(&mut self, id: &str) -> bool {
        let index = self
            .running_order()
            .iter()
            .position(|s| s == id)
            .filter(|&index| index > 0);
        if let Some(index) = index {
            self.swap(index - 1, index);
            true
        } else {
            false
        }
    }

    fn play(&mut self, id: &str) -> bool {
        if self.current_song_id().map(|cur| cur == id).unwrap_or(false) {
            return false;
        }
        let max = self.running_order().len();
        if let Some(mut index) = self.running_order().iter().position(|s| s == id) {
            if self.is_shuffled() && self.position.is_none() {
                // Hacky fix for now if we reach this state
                self.running_order_mut().swap(index, 0);
                index = 0;
            }
            self.position = Some(self.position.unwrap_or_default().update_into(index, max));
            self.is_playing = true;
            true
        } else {
            false
        }
    }

    fn stop(&mut self) {
        self.position = None;
        self.is_playing = false;
    }

    fn play_index(&mut self, index: usize) -> String {
        // Assumes index is in running order
        let len = self.running_order().len();
        self.is_playing = true;
        self.position = Some(self.position.unwrap_or_default().update_into(index, len));
        self.running_order()[index].clone()
    }

    fn play_next(&mut self) -> Option<String> {
        self.next_index().map(|i| self.play_index(i))
    }

    fn next_index(&self) -> Option<usize> {
        let len = self.running_order().len();
        self.position.and_then(|p| match self.repeat {
            RepeatMode::Song => Some(p.index),
            RepeatMode::Playlist => Some((p.index + 1) % len),
            RepeatMode::None => Some(p.index + 1).filter(|&i| i < len),
        })
    }

    fn play_prev(&mut self) -> Option<String> {
        self.prev_index().map(|i| self.play_index(i))
    }

    fn prev_index(&self) -> Option<usize> {
        let len = self.running_order().len();
        self.position.and_then(|p| match self.repeat {
            RepeatMode::Song => Some(p.index),
            RepeatMode::Playlist => Some((if p.index == 0 { len } else { p.index }) - 1),
            RepeatMode::None => Some(p.index).filter(|&i| i > 0).map(|i| i - 1),
        })
    }

    fn toggle_play(&mut self) -> Option<bool> {
        if self.position.is_some() {
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
            let id = self.current_song_id().cloned();
            self.running_order_shuffled = None;
            if let Some(id) = id {
                self.play(&id);
            }
        }
    }

    pub fn exhausted(&self) -> bool {
        self.position
            .map(|pos| pos.index + RANGE_SIZE >= self.running_order().len() - 1)
            .unwrap_or(false)
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            indexed_songs: HashMap::new(),
            running_order: VecDeque::with_capacity(QUEUE_DEFAULT_SIZE),
            running_order_shuffled: None,
            position: None,
            source: None,
            current_batch: None,
            repeat: RepeatMode::None,
            is_playing: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PlaybackAction {
    TogglePlay,
    Play,
    Pause,
    Stop,
    SetRepeatMode(RepeatMode),
    ToggleRepeat,
    ToggleShuffle,
    Seek(u32),
    SyncSeek(u32),
    Load(String),
    LoadSongs(Option<PlaylistSource>, Vec<SongDescription>),
    LoadPagedSongs(Option<PlaylistSource>, SongBatch),
    Next,
    Previous,
    Queue(Vec<SongDescription>),
    QueuePaged(SongBatch),
    Dequeue(String),
}

impl From<PlaybackAction> for AppAction {
    fn from(playback_action: PlaybackAction) -> Self {
        Self::PlaybackAction(playback_action)
    }
}

#[derive(Clone, Debug)]
pub enum PlaybackEvent {
    PlaybackPaused,
    PlaybackResumed,
    RepeatModeChanged(RepeatMode),
    TrackSeeked(u32),
    SeekSynced(u32),
    TrackChanged(String),
    ShuffleChanged,
    PlaylistChanged,
    PlaybackStopped,
}

#[derive(Clone, Copy, Debug)]
pub enum RepeatMode {
    Song,
    Playlist,
    None,
}

impl From<PlaybackEvent> for AppEvent {
    fn from(playback_event: PlaybackEvent) -> Self {
        Self::PlaybackEvent(playback_event)
    }
}

fn make_events(opt_events: Vec<Option<PlaybackEvent>>) -> Vec<PlaybackEvent> {
    opt_events.into_iter().flatten().collect()
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
            PlaybackAction::ToggleRepeat => {
                self.repeat = match self.repeat {
                    RepeatMode::Song => RepeatMode::None,
                    RepeatMode::Playlist => RepeatMode::Song,
                    RepeatMode::None => RepeatMode::Playlist,
                };
                vec![PlaybackEvent::RepeatModeChanged(self.repeat)]
            }
            PlaybackAction::SetRepeatMode(mode) => {
                self.repeat = mode;
                vec![PlaybackEvent::RepeatModeChanged(self.repeat)]
            }
            PlaybackAction::ToggleShuffle => {
                self.toggle_shuffle();
                vec![
                    PlaybackEvent::PlaylistChanged,
                    PlaybackEvent::ShuffleChanged,
                ]
            }
            PlaybackAction::Next => {
                let old_position = self.position;
                if let Some(id) = self.play_next() {
                    make_events(vec![
                        Some(PlaybackEvent::TrackChanged(id)),
                        Some(PlaybackEvent::PlaybackResumed),
                        Some(PlaybackEvent::PlaylistChanged)
                            .filter(|_| Position::has_range_moved(old_position, self.position)),
                    ])
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
                let old_position = self.position;
                if let Some(id) = self.play_prev() {
                    make_events(vec![
                        Some(PlaybackEvent::TrackChanged(id)),
                        Some(PlaybackEvent::PlaybackResumed),
                        Some(PlaybackEvent::PlaylistChanged)
                            .filter(|_| Position::has_range_moved(old_position, self.position)),
                    ])
                } else {
                    vec![]
                }
            }
            PlaybackAction::Load(id) => {
                let old_position = self.position;
                if self.play(&id) {
                    make_events(vec![
                        Some(PlaybackEvent::TrackChanged(id)),
                        Some(PlaybackEvent::PlaybackResumed),
                        Some(PlaybackEvent::PlaylistChanged)
                            .filter(|_| Position::has_range_moved(old_position, self.position)),
                    ])
                } else {
                    vec![]
                }
            }
            PlaybackAction::LoadSongs(source, tracks) => {
                self.set_playlist(source, None, tracks);
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::LoadPagedSongs(source, SongBatch { songs, batch }) => {
                self.set_playlist(source, Some(batch), songs);
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::Queue(tracks) => {
                self.current_batch = None;
                for track in tracks {
                    self.queue(track);
                }
                vec![PlaybackEvent::PlaylistChanged]
            }
            PlaybackAction::QueuePaged(SongBatch { batch, songs }) => {
                self.current_batch = Some(batch);
                for song in songs {
                    self.queue(song);
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::app::models::AlbumRef;

    fn song(id: &str) -> SongDescription {
        SongDescription {
            id: id.to_string(),
            uri: "".to_string(),
            title: "Title".to_string(),
            artists: vec![],
            album: AlbumRef {
                id: "".to_string(),
                name: "".to_string(),
            },
            duration: 1000,
            art: None,
        }
    }

    impl PlaybackState {
        fn current_position(&self) -> Option<usize> {
            Some(self.position?.index)
        }

        fn song_ids(&self) -> Vec<&str> {
            self.songs().map(|s| &s.id[..]).collect()
        }
    }

    #[test]
    fn test_initial_state() {
        let state = PlaybackState::default();
        assert!(!state.is_playing());
        assert!(!state.is_shuffled());
        assert!(state.current_song().is_none());
        assert!(state.prev_song().is_none());
        assert!(state.next_song().is_none());
    }

    #[test]
    fn test_play_one() {
        let mut state = PlaybackState::default();
        state.queue(song("foo"));

        state.play("foo");
        assert!(state.is_playing());

        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("foo"));
        assert!(state.prev_song().is_none());
        assert!(state.next_song().is_none());

        state.toggle_play();
        assert!(!state.is_playing());
    }

    #[test]
    fn test_queue() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));

        assert_eq!(state.songs().count(), 3);

        state.play("2");

        state.queue(song("4"));
        assert_eq!(state.songs().count(), 4);
    }

    #[test]
    fn test_play_multiple() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));
        assert_eq!(state.songs().count(), 3);

        state.play("2");
        assert!(state.is_playing());

        assert_eq!(state.current_position(), Some(1));
        assert_eq!(state.prev_song().map(|s| &s.id[..]), Some("1"));
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("2"));
        assert_eq!(state.next_song().map(|s| &s.id[..]), Some("3"));

        state.toggle_play();
        assert!(!state.is_playing());

        state.play_next();
        assert!(state.is_playing());
        assert_eq!(state.current_position(), Some(2));
        assert_eq!(state.prev_song().map(|s| &s.id[..]), Some("2"));
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("3"));
        assert!(state.next_song().is_none());

        state.play_next();
        assert!(state.is_playing());
        assert_eq!(state.current_position(), Some(2));
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("3"));

        state.play_prev();
        state.play_prev();
        assert!(state.is_playing());
        assert_eq!(state.current_position(), Some(0));
        assert!(state.prev_song().is_none());
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("1"));
        assert_eq!(state.next_song().map(|s| &s.id[..]), Some("2"));

        state.play_prev();
        assert!(state.is_playing());
        assert_eq!(state.current_position(), Some(0));
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("1"));
    }

    #[test]
    fn test_shuffle() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));
        state.queue(song("4"));

        assert_eq!(state.songs().count(), 4);

        state.play("2");
        assert_eq!(state.current_position(), Some(1));

        state.toggle_shuffle();
        assert!(state.is_shuffled());
        assert_eq!(state.current_position(), Some(0));

        state.play_next();
        assert_eq!(state.current_position(), Some(1));

        state.toggle_shuffle();
        assert!(!state.is_shuffled());

        let ids = state.song_ids();
        assert_eq!(ids, vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn test_shuffle_queue() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));

        state.toggle_shuffle();
        assert!(state.is_shuffled());

        state.queue(song("4"));

        state.toggle_shuffle();
        assert!(!state.is_shuffled());

        let ids = state.song_ids();
        assert_eq!(ids, vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn test_move() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));

        state.play("2");
        assert!(state.is_playing());

        state.move_down("1");
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("2"));
        let ids = state.song_ids();
        assert_eq!(ids, vec!["2", "1", "3"]);

        state.move_down("2");
        state.move_down("2");
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("2"));
        let ids = state.song_ids();
        assert_eq!(ids, vec!["1", "3", "2"]);

        state.move_down("2");
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("2"));
        let ids = state.song_ids();
        assert_eq!(ids, vec!["1", "3", "2"]);

        state.move_up("2");

        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("2"));
        let ids = state.song_ids();
        assert_eq!(ids, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_dequeue_last() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));

        state.play("3");
        assert!(state.is_playing());

        state.dequeue("3");
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("2"));
    }

    #[test]
    fn test_dequeue_a_few_songs() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));
        state.queue(song("4"));
        state.queue(song("5"));
        state.queue(song("6"));

        state.play("5");
        assert!(state.is_playing());

        state.dequeue("1");
        state.dequeue("2");
        state.dequeue("3");
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("5"));
    }

    #[test]
    fn test_dequeue_all() {
        let mut state = PlaybackState::default();
        state.queue(song("3"));

        state.play("3");
        assert!(state.is_playing());

        state.dequeue("3");
        assert_eq!(state.current_song().map(|s| &s.id[..]), None);
    }
}
