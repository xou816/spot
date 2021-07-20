use gettextrs::*;

use crate::app::credentials;
use crate::app::state::{LoginAction, TryLoginAction};
use crate::app::{ActionDispatcher, AppAction};

pub struct LoginModel {
    dispatcher: Box<dyn ActionDispatcher>,
}

impl LoginModel {
    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }

    pub fn try_autologin(&self) -> bool {
        if let Ok(creds) = credentials::try_retrieve_credentials() {
            self.dispatcher.dispatch(
                LoginAction::TryLogin(TryLoginAction::Token {
                    username: creds.username,
                    token: creds.token,
                })
                .into(),
            );
            true
        } else {
            false
        }
    }

    pub fn save_for_autologin(&self, credentials: credentials::Credentials) {
        if credentials::save_credentials(credentials).is_err() {
            self.dispatcher
                .dispatch(AppAction::ShowNotification(gettext(
                    // translators: This notification shows up right after login if the password could not be stored in the keyring (that is, GNOME's keyring aka seahorse, or any other libsecret compliant secret store).
                    "Could not save password. Make sure the session keyring is unlocked.",
                )));
        }
    }

    pub fn login(&self, username: String, password: String) {
        self.dispatcher.dispatch(
            LoginAction::TryLogin(TryLoginAction::Password { username, password }).into(),
        );
    }
}
