use crate::app::credentials;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum LoginAction {
    TryLogin(String, String),
    SetLoginSuccess(credentials::Credentials),
    SetLoginFailure,
    RefreshToken,
    SetRefreshedToken(String),
    Logout,
}

impl Into<AppAction> for LoginAction {
    fn into(self) -> AppAction {
        AppAction::LoginAction(self)
    }
}

#[derive(Clone, Debug)]
pub enum LoginEvent {
    LoginStarted(String, String),
    LoginCompleted(credentials::Credentials),
    LoginFailed,
    FreshTokenRequested,
    LogoutCompleted,
}

impl Into<AppEvent> for LoginEvent {
    fn into(self) -> AppEvent {
        AppEvent::LoginEvent(self)
    }
}

pub struct LoginState {
    pub user: Option<String>,
}

impl Default for LoginState {
    fn default() -> Self {
        Self { user: None }
    }
}

impl UpdatableState for LoginState {
    type Action = LoginAction;
    type Event = AppEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            LoginAction::TryLogin(u, p) => vec![LoginEvent::LoginStarted(u, p).into()],
            LoginAction::SetLoginSuccess(credentials) => {
                self.user = Some(credentials.username.clone());
                vec![LoginEvent::LoginCompleted(credentials).into()]
            }
            LoginAction::SetLoginFailure => vec![LoginEvent::LoginFailed.into()],
            LoginAction::RefreshToken => vec![LoginEvent::FreshTokenRequested.into()],
            LoginAction::SetRefreshedToken(_) => vec![AppEvent::NotificationShown(
                "Connection refreshed".to_string(),
            )],
            LoginAction::Logout => {
                self.user = None;
                vec![LoginEvent::LogoutCompleted.into()]
            }
        }
    }
}
