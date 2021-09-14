use crate::app::models::{InsertionRange, SongBatch, SongDescription, SongList};
use crate::app::state::{AppAction, AppEvent, UpdatableState};
use crate::app::{BatchQuery, LazyRandomIndex, SongsSource};

const RANGE_SIZE: usize = 25;

#[derive(Debug)]
pub struct PlaybackState {
    index: LazyRandomIndex,
    songs: SongList,
    position: Option<usize>,
    next_position: Option<usize>,
    source: Option<SongsSource>,
    repeat: RepeatMode,
    is_playing: bool,
    is_shuffled: bool,
}

impl PlaybackState {
    pub fn is_playing(&self) -> bool {
        self.is_playing && self.position.is_some()
    }

    pub fn is_shuffled(&self) -> bool {
        self.is_shuffled
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat
    }

    pub fn next_query(&self) -> Option<BatchQuery> {
        let next_index = self.index.get(self.next_index()?)?;
        let (batch, has_batch) = self.songs.has_batch_for(next_index);
        if !has_batch {
            let source = self.source.as_ref().cloned()?;
            Some(BatchQuery { source, batch })
        } else {
            None
        }
    }

    pub fn song(&self, id: &str) -> Option<&SongDescription> {
        self.songs.get(id)
    }

    pub fn len(&self) -> usize {
        self.songs.len()
    }

    pub fn songs(&self) -> impl Iterator<Item = &'_ SongDescription> + '_ {
        self.songs.iter()
    }

    fn index(&self, i: usize) -> Option<&SongDescription> {
        if self.is_shuffled {
            self.songs.index(self.index.get(i)?)
        } else {
            self.songs.index(i)
        }
    }

    pub fn current_song_id(&self) -> Option<&String> {
        Some(&self.index(self.position?).as_ref()?.id)
    }

    pub fn current_song(&self) -> Option<&SongDescription> {
        self.index(self.position?)
    }

    pub fn prev_song(&self) -> Option<&SongDescription> {
        self.prev_index().and_then(|i| self.index(i))
    }

    pub fn next_song(&self) -> Option<&SongDescription> {
        self.next_index().and_then(|i| self.index(i))
    }

    fn set_source(&mut self, source: Option<SongsSource>) {
        self.songs = SongList::new_sized(2 * RANGE_SIZE);
        self.source = source;
        self.index = Default::default();
        self.position = None;
    }

    fn add_batch(&mut self, song_batch: SongBatch) -> Option<InsertionRange> {
        let SongBatch { songs, batch } = song_batch;
        let range = self.songs.add(SongBatch { songs, batch });
        self.index.resize(self.songs.len());
        range
    }

    pub fn queue(&mut self, track: SongDescription) {
        self.songs.append(vec![track]);
        self.index.grow(self.songs.len());
    }

    pub fn dequeue(&mut self, id: &str) {
        let position = self.songs.iter_ids_from(0).position(|s| s == id);
        self.songs.remove(&[id.to_string()]);
        let new_len = self.songs.len();
        self.position = self
            .position
            .filter(|_| new_len > 0)
            .and_then(|p| Some(if p > 0 && p >= position? { p - 1 } else { p }));
        self.index.shrink(new_len);
    }

    fn swap(&mut self, index: usize, other_index: usize) {
        let len = self.songs.len();
        self.songs.swap(index, other_index);
        self.position = self
            .position
            .map(|position| match position {
                i if i == index => other_index,
                i if i == other_index => index,
                _ => position,
            })
            .map(|p| usize::min(p, len - 1))
    }

    pub fn move_down(&mut self, id: &str) -> Option<usize> {
        let index = self.songs.iter_ids_from(0).position(|s| s == id)?;
        self.swap(index, index + 1);
        Some(index)
    }

    pub fn move_up(&mut self, id: &str) -> Option<usize> {
        let index = self
            .songs
            .iter_ids_from(0)
            .position(|s| s == id)
            .filter(|&index| index > 0)?;
        self.swap(index - 1, index);
        Some(index)
    }

    fn play(&mut self, id: &str) -> bool {
        if self.current_song_id().map(|cur| cur == id).unwrap_or(false) {
            return false;
        }

        let found_index = self.songs.iter_ids_from(0).position(|s| &s[..] == id);

        if let Some(index) = found_index {
            if self.is_shuffled {
                self.index.reset_picking_first(index);
                self.play_index(0);
            } else {
                self.play_index(index);
            }
            true
        } else {
            false
        }
    }

    fn stop(&mut self) {
        self.position = None;
        self.is_playing = false;
    }

    fn play_index(&mut self, index: usize) -> Option<&String> {
        self.is_playing = true;
        self.position.replace(index);
        self.index.next_until(index + 1);
        self.current_song_id()
    }

    fn play_next(&mut self) -> Option<&String> {
        self.next_index().and_then(move |i| self.play_index(i))
    }

    fn next_index(&self) -> Option<usize> {
        let len = self.songs.len();
        self.position.and_then(|p| match self.repeat {
            RepeatMode::Song => Some(p),
            RepeatMode::Playlist => Some((p + 1) % len),
            RepeatMode::None => Some(p + 1).filter(|&i| i < len),
        })
    }

    fn play_prev(&mut self) -> Option<&String> {
        self.prev_index().and_then(move |i| self.play_index(i))
    }

    fn prev_index(&self) -> Option<usize> {
        let len = self.songs.len();
        self.position.and_then(|p| match self.repeat {
            RepeatMode::Song => Some(p),
            RepeatMode::Playlist => Some((if p == 0 { len } else { p }) - 1),
            RepeatMode::None => Some(p).filter(|&i| i > 0).map(|i| i - 1),
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
        self.is_shuffled = !self.is_shuffled;
        let old = self.position.replace(0).unwrap_or(0);
        self.index.reset_picking_first(old);
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            index: LazyRandomIndex::default(),
            songs: SongList::new_sized(2 * RANGE_SIZE),
            position: None,
            next_position: None,
            source: None,
            repeat: RepeatMode::None,
            is_playing: false,
            is_shuffled: false,
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
    LoadSongs(Vec<SongDescription>),
    LoadPagedSongs(SongsSource, SongBatch),
    Next,
    Previous,
    Queue(Vec<SongDescription>),
    Dequeue(String),
}

impl From<PlaybackAction> for AppAction {
    fn from(playback_action: PlaybackAction) -> Self {
        Self::PlaybackAction(playback_action)
    }
}

#[derive(Clone, Debug)]
pub enum PlaylistChange {
    Reset,
    InsertedAt(usize, usize),
    AppendedAt(usize),
    MovedUp(usize),
    MovedDown(usize),
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
    PlaylistChanged(PlaylistChange),
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
                vec![PlaybackEvent::ShuffleChanged]
            }
            PlaybackAction::Next => {
                if let Some(id) = self.play_next().cloned() {
                    make_events(vec![
                        Some(PlaybackEvent::TrackChanged(id)),
                        Some(PlaybackEvent::PlaybackResumed),
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
                if let Some(id) = self.play_prev().cloned() {
                    make_events(vec![
                        Some(PlaybackEvent::TrackChanged(id)),
                        Some(PlaybackEvent::PlaybackResumed),
                    ])
                } else {
                    vec![]
                }
            }
            PlaybackAction::Load(id) => {
                if self.play(&id) {
                    make_events(vec![
                        Some(PlaybackEvent::TrackChanged(id)),
                        Some(PlaybackEvent::PlaybackResumed),
                    ])
                } else {
                    vec![]
                }
            }
            PlaybackAction::LoadSongs(tracks) => {
                self.set_source(None);
                for track in tracks {
                    self.queue(track);
                }
                vec![PlaybackEvent::PlaylistChanged(PlaylistChange::Reset)]
            }
            PlaybackAction::LoadPagedSongs(source, batch)
                if Some(&source) == self.source.as_ref() =>
            {
                if let Some(InsertionRange(a, b)) = self.add_batch(batch) {
                    vec![PlaybackEvent::PlaylistChanged(PlaylistChange::InsertedAt(
                        a, b,
                    ))]
                } else {
                    vec![]
                }
            }
            PlaybackAction::LoadPagedSongs(source, batch)
                if Some(&source) != self.source.as_ref() =>
            {
                self.set_source(Some(source));
                self.add_batch(batch);
                vec![PlaybackEvent::PlaylistChanged(PlaylistChange::Reset)]
            }
            PlaybackAction::Queue(tracks) => {
                let append_at = self.songs.partial_len();
                self.source = None;
                for track in tracks {
                    self.queue(track);
                }
                vec![PlaybackEvent::PlaylistChanged(PlaylistChange::AppendedAt(
                    append_at,
                ))]
            }
            PlaybackAction::Dequeue(id) => {
                self.dequeue(&id);
                vec![PlaybackEvent::PlaylistChanged(PlaylistChange::Reset)]
            }
            PlaybackAction::Seek(pos) => vec![PlaybackEvent::TrackSeeked(pos)],
            PlaybackAction::SyncSeek(pos) => vec![PlaybackEvent::SeekSynced(pos)],
            _ => vec![],
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
            track_number: 0,
        }
    }

    impl PlaybackState {
        fn current_position(&self) -> Option<usize> {
            self.position
        }

        fn song_ids(&self) -> Vec<&str> {
            self.songs().map(|s| &s.id[..]).collect()
        }

        fn song_id(&self) -> Option<&str> {
            self.current_song().map(|s| &s.id[..])
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

        assert_eq!(state.song_id(), Some("foo"));
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
        assert_eq!(state.song_id(), Some("2"));
        assert_eq!(state.next_song().map(|s| &s.id[..]), Some("3"));

        state.toggle_play();
        assert!(!state.is_playing());

        state.play_next();
        assert!(state.is_playing());
        assert_eq!(state.current_position(), Some(2));
        assert_eq!(state.prev_song().map(|s| &s.id[..]), Some("2"));
        assert_eq!(state.song_id(), Some("3"));
        assert!(state.next_song().is_none());

        state.play_next();
        assert!(state.is_playing());
        assert_eq!(state.current_position(), Some(2));
        assert_eq!(state.song_id(), Some("3"));

        state.play_prev();
        state.play_prev();
        assert!(state.is_playing());
        assert_eq!(state.current_position(), Some(0));
        assert!(state.prev_song().is_none());
        assert_eq!(state.song_id(), Some("1"));
        assert_eq!(state.next_song().map(|s| &s.id[..]), Some("2"));

        state.play_prev();
        assert!(state.is_playing());
        assert_eq!(state.current_position(), Some(0));
        assert_eq!(state.song_id(), Some("1"));
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
        assert_eq!(state.song_id(), Some("2"));
        let ids = state.song_ids();
        assert_eq!(ids, vec!["2", "1", "3"]);

        state.move_down("2");
        state.move_down("2");
        assert_eq!(state.song_id(), Some("2"));
        let ids = state.song_ids();
        assert_eq!(ids, vec!["1", "3", "2"]);

        state.move_down("2");
        assert_eq!(state.song_id(), Some("2"));
        let ids = state.song_ids();
        assert_eq!(ids, vec!["1", "3", "2"]);

        state.move_up("2");

        assert_eq!(state.song_id(), Some("2"));
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
        assert_eq!(state.song_id(), Some("2"));
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
        assert_eq!(state.song_id(), Some("5"));
    }

    #[test]
    fn test_dequeue_all() {
        let mut state = PlaybackState::default();
        state.queue(song("3"));

        state.play("3");
        assert!(state.is_playing());

        state.dequeue("3");
        assert_eq!(state.song_id(), None);
    }
}
