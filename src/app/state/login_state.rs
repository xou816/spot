use gettextrs::*;
use std::borrow::Cow;
use std::time::SystemTime;

use crate::app::credentials::Credentials;
use crate::app::models::PlaylistSummary;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum TryLoginAction {
    Password { username: String, password: String },
    Token { username: String, token: String },
}

#[derive(Clone, Debug)]
pub enum SetLoginSuccessAction {
    Password(Credentials),
    Token { username: String, token: String },
}

#[derive(Clone, Debug)]
pub enum LoginAction {
    ShowLogin,
    TryLogin(TryLoginAction),
    SetLoginSuccess(SetLoginSuccessAction),
    SetUserPlaylists(Vec<PlaylistSummary>),
    UpdateUserPlaylist(PlaylistSummary),
    PrependUserPlaylist(Vec<PlaylistSummary>),
    SetLoginFailure,
    RefreshToken,
    SetRefreshedToken {
        token: String,
        token_expiry_time: SystemTime,
    },
    Logout,
}

impl From<LoginAction> for AppAction {
    fn from(login_action: LoginAction) -> Self {
        Self::LoginAction(login_action)
    }
}

#[derive(Clone, Debug)]
pub enum LoginStartedEvent {
    Password { username: String, password: String },
    Token { username: String, token: String },
}

#[derive(Clone, Debug)]
pub enum LoginCompletedEvent {
    Password(Credentials),
    Token,
}

#[derive(Clone, Debug)]
pub enum LoginEvent {
    LoginShown,
    LoginStarted(LoginStartedEvent),
    LoginCompleted(LoginCompletedEvent),
    UserPlaylistsLoaded,
    LoginFailed,
    FreshTokenRequested,
    RefreshTokenCompleted {
        token: String,
        token_expiry_time: SystemTime,
    },
    LogoutCompleted,
}

impl From<LoginEvent> for AppEvent {
    fn from(login_event: LoginEvent) -> Self {
        Self::LoginEvent(login_event)
    }
}

#[derive(Default)]
pub struct LoginState {
    pub user: Option<String>,
    pub playlists: Vec<PlaylistSummary>,
}

impl UpdatableState for LoginState {
    type Action = LoginAction;
    type Event = AppEvent;

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.into_owned() {
            LoginAction::ShowLogin => vec![LoginEvent::LoginShown.into()],
            LoginAction::TryLogin(TryLoginAction::Password { username, password }) => {
                vec![
                    LoginEvent::LoginStarted(LoginStartedEvent::Password { username, password })
                        .into(),
                ]
            }
            LoginAction::TryLogin(TryLoginAction::Token { username, token }) => {
                vec![LoginEvent::LoginStarted(LoginStartedEvent::Token { username, token }).into()]
            }
            LoginAction::SetLoginSuccess(SetLoginSuccessAction::Password(creds)) => {
                self.user = Some(creds.username.clone());
                vec![LoginEvent::LoginCompleted(LoginCompletedEvent::Password(creds)).into()]
            }
            LoginAction::SetLoginSuccess(SetLoginSuccessAction::Token { username, .. }) => {
                self.user = Some(username);
                vec![LoginEvent::LoginCompleted(LoginCompletedEvent::Token).into()]
            }
            LoginAction::SetLoginFailure => vec![LoginEvent::LoginFailed.into()],
            LoginAction::RefreshToken => vec![LoginEvent::FreshTokenRequested.into()],
            LoginAction::SetRefreshedToken {
                token,
                token_expiry_time,
            } => {
                // translators: This notification is shown when, after some inactivity, the session is successfully restored. The user might have to repeat its last action.
                vec![
                    AppEvent::NotificationShown(gettext("Connection restored")),
                    LoginEvent::RefreshTokenCompleted {
                        token,
                        token_expiry_time,
                    }
                    .into(),
                ]
            }
            LoginAction::Logout => {
                self.user = None;
                vec![LoginEvent::LogoutCompleted.into()]
            }
            LoginAction::SetUserPlaylists(playlists) => {
                self.playlists = playlists;
                vec![LoginEvent::UserPlaylistsLoaded.into()]
            }
            LoginAction::UpdateUserPlaylist(PlaylistSummary { id, title }) => {
                if let Some(p) = self.playlists.iter_mut().find(|p| p.id == id) {
                    p.title = title;
                }
                vec![LoginEvent::UserPlaylistsLoaded.into()]
            }
            LoginAction::PrependUserPlaylist(mut summaries) => {
                summaries.append(&mut self.playlists);
                self.playlists = summaries;
                vec![LoginEvent::UserPlaylistsLoaded.into()]
            }
        }
    }
}
