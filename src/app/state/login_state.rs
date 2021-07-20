use gettextrs::*;

use crate::app::credentials;
use crate::app::models::PlaylistSummary;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum LoginAction {
    TryLogin { username: String, password: String },
    TryAutologin { username: String, token: String },
    SetLoginSuccess(credentials::Credentials),
    SetAutologinSuccess { username: String },
    SetUserPlaylists(Vec<PlaylistSummary>),
    SetLoginFailure,
    RefreshToken,
    SetRefreshedToken(String),
    Logout,
}

impl From<LoginAction> for AppAction {
    fn from(login_action: LoginAction) -> Self {
        Self::LoginAction(login_action)
    }
}

#[derive(Clone, Debug)]
pub enum LoginEvent {
    LoginStarted { username: String, password: String },
    AutologinStarted { username: String, token: String },
    LoginCompleted(credentials::Credentials),
    AutologinCompleted,
    UserPlaylistsLoaded,
    LoginFailed,
    FreshTokenRequested,
    LogoutCompleted,
}

impl From<LoginEvent> for AppEvent {
    fn from(login_event: LoginEvent) -> Self {
        Self::LoginEvent(login_event)
    }
}

pub struct LoginState {
    pub user: Option<String>,
    pub playlists: Vec<PlaylistSummary>,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            user: None,
            playlists: vec![],
        }
    }
}

impl UpdatableState for LoginState {
    type Action = LoginAction;
    type Event = AppEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            LoginAction::TryLogin { username, password } => {
                vec![LoginEvent::LoginStarted { username, password }.into()]
            }
            LoginAction::TryAutologin { username, token } => {
                vec![LoginEvent::AutologinStarted { username, token }.into()]
            }
            LoginAction::SetLoginSuccess(credentials) => {
                self.user = Some(credentials.username.clone());
                vec![LoginEvent::LoginCompleted(credentials).into()]
            }
            LoginAction::SetAutologinSuccess { username } => {
                self.user = Some(username);
                vec![LoginEvent::AutologinCompleted.into()]
            }
            LoginAction::SetLoginFailure => vec![LoginEvent::LoginFailed.into()],
            LoginAction::RefreshToken => vec![LoginEvent::FreshTokenRequested.into()],
            LoginAction::SetRefreshedToken(_) => {
                // translators: This notification is shown when, after some inactivity, the session is successfully restored. The user might have to repeat its last action.
                vec![AppEvent::NotificationShown(gettext("Connection restored"))]
            }
            LoginAction::Logout => {
                self.user = None;
                vec![LoginEvent::LogoutCompleted.into()]
            }
            LoginAction::SetUserPlaylists(playlists) => {
                self.playlists = playlists;
                vec![LoginEvent::UserPlaylistsLoaded.into()]
            }
        }
    }
}
