use gettextrs::*;
use std::borrow::Cow;
use url::Url;

use crate::app::models::PlaylistSummary;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum TryLoginAction {
    Restore,
    InitLogin,
    CompleteLogin,
}

#[derive(Clone, Debug)]
pub enum LoginAction {
    ShowLogin,
    OpenLoginUrl(Url),
    TryLogin(TryLoginAction),
    SetLoginSuccess(String),
    SetUserPlaylists(Vec<PlaylistSummary>),
    UpdateUserPlaylist(PlaylistSummary),
    PrependUserPlaylist(Vec<PlaylistSummary>),
    SetLoginFailure,
    RefreshToken,
    TokenRefreshed,
    Logout,
}

impl From<LoginAction> for AppAction {
    fn from(login_action: LoginAction) -> Self {
        Self::LoginAction(login_action)
    }
}

#[derive(Clone, Debug)]
pub enum LoginStartedEvent {
    Restore,
    InitLogin,
    CompleteLogin,
    OpenUrl(Url),
}

#[derive(Clone, Debug)]
pub enum LoginEvent {
    LoginShown,
    LoginStarted(LoginStartedEvent),
    LoginCompleted,
    UserPlaylistsLoaded,
    LoginFailed,
    FreshTokenRequested,
    RefreshTokenCompleted,
    LogoutCompleted,
}

impl From<LoginEvent> for AppEvent {
    fn from(login_event: LoginEvent) -> Self {
        Self::LoginEvent(login_event)
    }
}

#[derive(Default)]
pub struct LoginState {
    // Username
    pub user: Option<String>,
    // Playlists owned by the logged in user
    pub playlists: Vec<PlaylistSummary>,
}

impl UpdatableState for LoginState {
    type Action = LoginAction;
    type Event = AppEvent;

    // The login state has a lot of actions that just translate to events
    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        info!("update_with({:?})", action);
        match action.into_owned() {
            LoginAction::ShowLogin => vec![LoginEvent::LoginShown.into()],
            LoginAction::OpenLoginUrl(url) => {
                vec![LoginEvent::LoginStarted(LoginStartedEvent::OpenUrl(url)).into()]
            }
            LoginAction::TryLogin(TryLoginAction::Restore) => {
                vec![LoginEvent::LoginStarted(LoginStartedEvent::Restore).into()]
            }
            LoginAction::TryLogin(TryLoginAction::CompleteLogin) => {
                vec![LoginEvent::LoginStarted(LoginStartedEvent::CompleteLogin).into()]
            }
            LoginAction::SetLoginSuccess(username) => {
                self.user = Some(username);
                vec![LoginEvent::LoginCompleted.into()]
            }
            LoginAction::SetLoginFailure => vec![LoginEvent::LoginFailed.into()],
            LoginAction::RefreshToken => vec![LoginEvent::FreshTokenRequested.into()],
            LoginAction::TokenRefreshed => {
                // translators: This notification is shown when, after some inactivity, the session is successfully restored. The user might have to repeat its last action.
                vec![
                    AppEvent::NotificationShown(gettext("Connection restored")),
                    LoginEvent::RefreshTokenCompleted.into(),
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
            LoginAction::TryLogin(TryLoginAction::InitLogin) => {
                vec![LoginEvent::LoginStarted(LoginStartedEvent::InitLogin).into()]
            }
        }
    }
}
