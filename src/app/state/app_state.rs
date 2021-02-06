use crate::app::credentials;
use crate::app::models::*;
use crate::app::state::{BrowserAction, BrowserEvent, BrowserState, ScreenName, UpdatableState};
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};

#[derive(Clone, Debug)]
pub enum AppAction {
    TogglePlay,
    ToggleShuffle,
    Seek(u32),
    SyncSeek(u32),
    Load(String),
    LoadPlaylist(Vec<SongDescription>),
    Start,
    TryLogin(String, String),
    RefreshToken,
    SetRefreshedToken(String),
    SetLoginSuccess(credentials::Credentials),
    Logout,
    Next,
    Previous,
    BrowserAction(BrowserAction),
    ShowNotification(String),
    HideNotification,
}

impl AppAction {
    #[allow(non_snake_case)]
    pub fn ViewAlbum(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::Details(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewArtist(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::Artist(id)).into()
    }
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Started,
    TrackPaused,
    TrackResumed,
    TrackSeeked(u32),
    SeekSynced(u32),
    FreshTokenRequested,
    LoginStarted(String, String),
    LoginCompleted(credentials::Credentials),
    LogoutCompleted,
    TrackChanged(String),
    PlaylistChanged,
    BrowserEvent(BrowserEvent),
    NotificationShown(String),
    NotificationHidden,
}

pub struct ShuffledSongs {
    rng: SmallRng,
    internal_playlist: Vec<SongDescription>,
    playlist: Vec<SongDescription>,
    shuffled: bool,
}

impl ShuffledSongs {
    fn new(tracks: Vec<SongDescription>) -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            internal_playlist: tracks,
            playlist: vec![],
            shuffled: false,
        }
    }

    fn update(&mut self, tracks: Vec<SongDescription>, keep_index: Option<usize>) {
        self.internal_playlist = tracks;
        if self.shuffled {
            self.shuffle(keep_index);
        }
    }

    pub fn songs(&self) -> &Vec<SongDescription> {
        if self.shuffled {
            &self.playlist
        } else {
            &self.internal_playlist
        }
    }

    fn shuffle(&mut self, keep_index: Option<usize>) {
        let mut shuffled = self.internal_playlist.clone();
        let mut final_list = if let Some(index) = keep_index {
            vec![shuffled.remove(index)]
        } else {
            vec![]
        };
        shuffled.shuffle(&mut self.rng);
        final_list.append(&mut shuffled);
        self.playlist = final_list;
    }

    pub fn toggle_shuffle(&mut self, keep_index: Option<usize>) {
        if !self.shuffled {
            self.shuffle(keep_index);
        }
        self.shuffled = !self.shuffled;
    }
}

pub struct AppState {
    pub is_playing: bool,
    pub current_song_id: Option<String>,
    pub playlist: ShuffledSongs,
    pub browser_state: BrowserState,
    pub user: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            current_song_id: None,
            playlist: ShuffledSongs::new(vec![]),
            browser_state: BrowserState::new(),
            user: None,
        }
    }

    pub fn update_state(&mut self, message: AppAction) -> Vec<AppEvent> {
        match message {
            AppAction::TogglePlay => {
                if self.is_playing {
                    self.is_playing = false;
                    vec![AppEvent::TrackPaused]
                } else if self.current_song_id.is_some() {
                    self.is_playing = true;
                    vec![AppEvent::TrackResumed]
                } else {
                    vec![]
                }
            }
            AppAction::ToggleShuffle => {
                self.playlist.toggle_shuffle(self.song_index());
                vec![AppEvent::PlaylistChanged]
            }
            AppAction::Next => {
                let next = self.next_song().map(|s| s.id.clone());
                if next.is_some() {
                    self.is_playing = true;
                    self.current_song_id = next.clone();
                    vec![
                        AppEvent::TrackChanged(next.unwrap()),
                        AppEvent::TrackResumed,
                    ]
                } else {
                    vec![]
                }
            }
            AppAction::Previous => {
                let prev = self.prev_song().map(|s| s.id.clone());
                if prev.is_some() {
                    self.is_playing = true;
                    self.current_song_id = prev.clone();
                    vec![
                        AppEvent::TrackChanged(prev.unwrap()),
                        AppEvent::TrackResumed,
                    ]
                } else {
                    vec![]
                }
            }
            AppAction::Load(id) => {
                self.is_playing = true;
                self.current_song_id = Some(id.clone());
                vec![AppEvent::TrackChanged(id), AppEvent::TrackResumed]
            }
            AppAction::LoadPlaylist(tracks) => {
                self.playlist.update(tracks, None);
                vec![AppEvent::PlaylistChanged]
            }
            AppAction::SetLoginSuccess(credentials) => {
                self.user = Some(credentials.username.clone());
                vec![AppEvent::LoginCompleted(credentials)]
            }
            AppAction::RefreshToken => vec![AppEvent::FreshTokenRequested],
            AppAction::SetRefreshedToken(_) => vec![AppEvent::NotificationShown(
                "Connection refreshed".to_string(),
            )],
            AppAction::Logout => {
                self.user = None;
                vec![AppEvent::LogoutCompleted]
            }
            AppAction::Seek(pos) => vec![AppEvent::TrackSeeked(pos)],
            AppAction::SyncSeek(pos) => vec![AppEvent::SeekSynced(pos)],
            AppAction::Start => vec![AppEvent::Started],
            AppAction::TryLogin(u, p) => vec![AppEvent::LoginStarted(u, p)],
            AppAction::BrowserAction(a) => self
                .browser_state
                .update_with(a)
                .into_iter()
                .map(AppEvent::BrowserEvent)
                .collect(),
            AppAction::ShowNotification(c) => vec![AppEvent::NotificationShown(c)],
            AppAction::HideNotification => vec![AppEvent::NotificationHidden],
        }
    }

    fn song_index(&self) -> Option<usize> {
        self.current_song_id
            .as_ref()
            .and_then(|id| self.playlist.songs().iter().position(|song| song.id == *id))
    }

    pub fn current_song(&self) -> Option<SongDescription> {
        if let Some(current_song_id) = self.current_song_id.as_ref() {
            self.playlist
                .songs()
                .iter()
                .find(|song| song.id == *current_song_id)
                .cloned()
        } else {
            None
        }
    }

    pub fn prev_song(&self) -> Option<&SongDescription> {
        self.current_song_id.as_ref().and_then(|id| {
            self.playlist
                .songs()
                .iter()
                .take_while(|&song| song.id != *id)
                .last()
        })
    }

    pub fn next_song(&self) -> Option<&SongDescription> {
        self.current_song_id.as_ref().and_then(|id| {
            self.playlist
                .songs()
                .iter()
                .skip_while(|&song| song.id != *id)
                .nth(1)
        })
    }
}
