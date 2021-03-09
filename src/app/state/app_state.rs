use crate::app::credentials;
use crate::app::state::{
    browser_state::{BrowserAction, BrowserEvent, BrowserState},
    playback_state::{PlaybackAction, PlaybackEvent, PlaybackState},
    selection_state::{SelectionAction, SelectionEvent, SelectionState},
    ScreenName, UpdatableState,
};

#[derive(Clone, Debug)]
pub enum AppAction {
    PlaybackAction(PlaybackAction),
    BrowserAction(BrowserAction),
    SelectionAction(SelectionAction),
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
    SelectionEvent(SelectionEvent),
    Started,
    Raised,
    FreshTokenRequested,
    LoginStarted(String, String),
    LoginCompleted(credentials::Credentials),
    LogoutCompleted,
    NotificationShown(String),
    NotificationHidden,
    NowPlayingShown,
}

pub struct AppState {
    pub playback: PlaybackState,
    pub browser: BrowserState,
    pub user: Option<String>,
    pub selection: SelectionState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            playback: Default::default(),
            browser: BrowserState::new(),
            user: None,
            selection: Default::default(),
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
            AppAction::SelectionAction(a) => self
                .selection
                .update_with(a)
                .into_iter()
                .map(AppEvent::SelectionEvent)
                .collect(),
        }
    }
}
