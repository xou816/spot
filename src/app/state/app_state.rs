use crate::app::credentials;
use crate::app::models::*;
use crate::app::state::{BrowserAction, BrowserEvent, BrowserState, ScreenName, UpdatableState};
use super::PlayQueue;

#[derive(Clone, Debug)]
pub enum AppAction {
    TogglePlay,
    ToggleShuffle,
    Seek(u32),
    SyncSeek(u32),
    Load(String),
    LoadPlaylist(Vec<SongDescription>),
    Start,
    Raise,
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
    ViewNowPlaying,
}

impl AppAction {
    #[allow(non_snake_case)]
    pub fn ViewAlbum(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::AlbumDetails(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewArtist(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::Artist(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewPlaylist(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::PlaylistDetails(id)).into()
    }
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Started,
    Raised,
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
    NowPlayingShown,
}

pub struct AppState {
    pub is_playing: bool,
    pub current_song_id: Option<String>,
    pub playlist: PlayQueue,
    pub browser_state: BrowserState,
    pub user: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            current_song_id: None,
            playlist: PlayQueue::new(&[]),
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
                //self.playlist.toggle_shuffle(self.song_index());
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
                self.playlist.set_tracks(&tracks, None);
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
            AppAction::ViewNowPlaying => vec![AppEvent::NowPlayingShown],
            AppAction::Raise => vec![AppEvent::Raised],
        }
    }

    pub fn current_song(&self) -> Option<SongDescription> {
        self.current_song_id.as_ref().and_then(|current_song_id| {
            self.playlist
                .song(current_song_id)
                .cloned()
        })
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
