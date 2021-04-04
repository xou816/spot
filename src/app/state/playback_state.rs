use rand::{rngs::SmallRng, seq::SliceRandom, RngCore, SeedableRng};
use std::collections::{HashMap, VecDeque};

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
    running_order: VecDeque<String>,
    running_order_shuffled: Option<VecDeque<String>>,
    pub position: Option<usize>,
    pub pagination: Option<Pagination<PlaylistSource>>,
    is_playing: bool,
}

impl PlaybackState {
    pub fn is_playing(&self) -> bool {
        self.is_playing && self.position.is_some()
    }

    pub fn is_shuffled(&self) -> bool {
        self.running_order_shuffled.is_some()
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

    pub fn songs<'s>(&'s self) -> impl Iterator<Item = &'s SongDescription> + 's {
        let indexed = &self.indexed_songs;
        self.running_order()
            .iter()
            .filter_map(move |id| indexed.get(id))
    }

    pub fn current_song_id(&self) -> Option<&String> {
        self.position.map(|pos| &self.running_order()[pos])
    }

    pub fn current_song(&self) -> Option<&SongDescription> {
        self.current_song_id().and_then(|id| self.song(id))
    }

    pub fn prev_song(&self) -> Option<&SongDescription> {
        self.position
            .filter(|pos| *pos > 0)
            .map(|pos| &self.running_order()[pos - 1])
            .and_then(|id| self.song(id))
    }

    pub fn next_song(&self) -> Option<&SongDescription> {
        let len = self.running_order().len();
        self.position
            .filter(|pos| *pos + 1 < len)
            .map(|pos| &self.running_order()[pos + 1])
            .and_then(|id| self.song(id))
    }

    fn index_tracks(
        tracks: impl Iterator<Item = SongDescription>,
    ) -> HashMap<String, SongDescription> {
        let mut map: HashMap<String, SongDescription> =
            HashMap::with_capacity(tracks.size_hint().1.unwrap_or_else(Self::max_size));
        for track in tracks {
            map.insert(track.id.clone(), track);
        }
        map
    }

    fn shuffle(&mut self) {
        let mut to_shuffle: Vec<String> = self
            .running_order
            .iter()
            .filter(|&id| Some(id) != self.current_song_id())
            .cloned()
            .collect();
        let mut final_list: VecDeque<String> =
            self.current_song_id().cloned().into_iter().collect();
        to_shuffle.shuffle(&mut self.rng);
        final_list.append(&mut to_shuffle.into());
        self.position = self.position.map(|_| 0);
        self.running_order_shuffled = Some(final_list);
    }

    fn set_playlist(&mut self, source: Option<PlaylistSource>, tracks: Vec<SongDescription>) {
        self.pagination = source.map(|source| {
            let mut p = Pagination::new(source, Self::max_size());
            p.reset_count(tracks.len());
            p
        });
        self.running_order = tracks.iter().map(|t| t.id.clone()).collect();
        self.indexed_songs = Self::index_tracks(tracks.into_iter());
        if self.is_shuffled() {
            self.shuffle();
        }
    }

    pub fn queue(&mut self, track: SongDescription) {
        if !self.indexed_songs.contains_key(&track.id) {
            self.pagination = None;

            //if self.running_order.len() == 2 * Self::max_size() {
            //    self.pop_front();
            //}

            self.running_order.push_back(track.id.clone());
            if let Some(shuffled) = self.running_order_shuffled.as_mut() {
                let next = (self.rng.next_u32() as usize) % (shuffled.len() - 1);
                shuffled.insert(next + 1, track.id.clone());
            }

            self.indexed_songs.insert(track.id.clone(), track);
        }
    }

    fn pop_front(&mut self) {
        if let (Some(normal_front), Some(shuffled_front)) = (
            self.running_order.pop_front(),
            self.running_order_shuffled
                .as_mut()
                .and_then(|s| s.pop_front()),
        ) {
            if normal_front == shuffled_front {
                self.indexed_songs.remove(&normal_front);
            }
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

    fn swap(&mut self, index: usize, other_index: usize) {
        let running_order = self.running_order_mut();
        running_order.swap(index, other_index);
        if let Some(position) = self.position.as_mut() {
            if index == *position {
                *position = other_index;
            } else if other_index == *position {
                *position = index;
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

    fn play(&mut self, id: &str) {
        let position = self.songs().position(|s| s.id == id);
        self.position = position;
        self.is_playing = true;
    }

    fn stop(&mut self) {
        self.position = None;
        self.is_playing = false;
    }

    fn play_next(&mut self) -> Option<String> {
        let len = self.running_order().len();
        let next = self.position.filter(|&p| p + 1 < len).map(|p| p + 1);
        if let Some(next) = next {
            self.position = Some(next);
            Some(self.running_order()[next].clone())
        } else {
            None
        }
    }

    fn play_prev(&mut self) -> Option<String> {
        let prev = self.position.filter(|&p| p > 0).map(|p| p - 1);
        if let Some(prev) = prev {
            self.position = Some(prev);
            Some(self.running_order()[prev].clone())
        } else {
            None
        }
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

    pub fn source(&self) -> Option<&PlaylistSource> {
        self.pagination.as_ref().map(|p| &p.data)
    }

    pub fn max_size() -> usize {
        50
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            indexed_songs: HashMap::new(),
            running_order: VecDeque::with_capacity(2 * Self::max_size()),
            running_order_shuffled: None,
            position: None,
            pagination: None,
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
                if self.current_song_id() != Some(&id) {
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::app::models::AlbumRef;

    fn song(id: &str) -> SongDescription {
        SongDescription {
            id: id.to_string(),
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
    fn test_play_multiple() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));

        state.play("2");
        assert!(state.is_playing());

        assert_eq!(state.position, Some(1));
        assert_eq!(state.prev_song().map(|s| &s.id[..]), Some("1"));
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("2"));
        assert_eq!(state.next_song().map(|s| &s.id[..]), Some("3"));

        state.play_next();
        assert!(state.is_playing());
        assert_eq!(state.position, Some(2));
        assert_eq!(state.prev_song().map(|s| &s.id[..]), Some("2"));
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("3"));
        assert!(state.next_song().is_none());

        state.play_next();
        assert!(state.is_playing());
        assert_eq!(state.position, Some(2));
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("3"));

        state.play_prev();
        state.play_prev();
        assert!(state.is_playing());
        assert_eq!(state.position, Some(0));
        assert!(state.prev_song().is_none());
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("1"));
        assert_eq!(state.next_song().map(|s| &s.id[..]), Some("2"));

        state.play_prev();
        assert!(state.is_playing());
        assert_eq!(state.position, Some(0));
        assert_eq!(state.current_song().map(|s| &s.id[..]), Some("1"));
    }

    #[test]
    fn test_shuffle() {
        let mut state = PlaybackState::default();
        state.queue(song("1"));
        state.queue(song("2"));
        state.queue(song("3"));
        state.queue(song("4"));

        state.play("2");
        assert_eq!(state.position, Some(1));

        state.toggle_shuffle();
        assert!(state.is_shuffled());
        assert_eq!(state.position, Some(0));

        state.play_next();
        assert_eq!(state.position, Some(1));

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
}
