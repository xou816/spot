use crate::app::credentials;
use crate::app::models::SongDescription;
use crate::app::state::{
    browser_state::{BrowserAction, BrowserEvent, BrowserState},
    playback_state::{PlaybackAction, PlaybackEvent, PlaybackState},
    ScreenName, UpdatableState,
};

#[derive(Clone, Debug)]
pub enum AppAction {
    PlaybackAction(PlaybackAction),
    BrowserAction(BrowserAction),
    Start,
    Raise,
    TryLogin(String, String),
    RefreshToken,
    SetRefreshedToken(String),
    SetLoginSuccess(credentials::Credentials),
    Logout,
    ShowNotification(String),
    HideNotification,
    ViewNowPlaying,
    ToggleSelectionMode,
    Select(SongDescription),
    Deselect(String),
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
    PlaybackEvent(PlaybackEvent),
    BrowserEvent(BrowserEvent),
    Started,
    Raised,
    FreshTokenRequested,
    LoginStarted(String, String),
    LoginCompleted(credentials::Credentials),
    LogoutCompleted,
    NotificationShown(String),
    NotificationHidden,
    NowPlayingShown,
    SelectionModeChanged(bool),
    Selected(String),
    Deselected(String),
}

pub struct AppState {
    pub playback: PlaybackState,
    pub browser: BrowserState,
    pub user: Option<String>,
    pub selection: Option<Vec<SongDescription>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            playback: Default::default(),
            browser: BrowserState::new(),
            user: None,
            selection: None,
        }
    }

    pub fn update_state(&mut self, message: AppAction) -> Vec<AppEvent> {
        match message {
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
            AppAction::Start => vec![AppEvent::Started],
            AppAction::TryLogin(u, p) => vec![AppEvent::LoginStarted(u, p)],
            AppAction::ShowNotification(c) => vec![AppEvent::NotificationShown(c)],
            AppAction::HideNotification => vec![AppEvent::NotificationHidden],
            AppAction::ViewNowPlaying => vec![AppEvent::NowPlayingShown],
            AppAction::Raise => vec![AppEvent::Raised],
            AppAction::ToggleSelectionMode => {
                if self.selection.is_some() {
                    self.selection = None;
                    vec![AppEvent::SelectionModeChanged(false)]
                } else {
                    self.selection = Some(vec![]);
                    vec![AppEvent::SelectionModeChanged(true)]
                }
            }
            AppAction::Select(track) => {
                if let Some(selection) = self.selection.as_mut() {
                    let id = track.id.clone();
                    selection.push(track);
                    vec![AppEvent::Selected(id)]
                } else {
                    vec![]
                }
            }
            AppAction::Deselect(id) => {
                if let Some(selection) = self.selection.as_mut() {
                    selection.retain(|t| &t.id != &id);
                    vec![AppEvent::Deselected(id)]
                } else {
                    vec![]
                }
            }
            AppAction::PlaybackAction(a) => self
                .playback
                .update_with(a)
                .into_iter()
                .map(AppEvent::PlaybackEvent)
                .collect(),
            AppAction::BrowserAction(a) => self
                .browser
                .update_with(a)
                .into_iter()
                .map(AppEvent::BrowserEvent)
                .collect(),
        }
    }
}
