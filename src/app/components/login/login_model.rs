use std::time::SystemTime;

use gettextrs::*;

use crate::app::credentials::Credentials;
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
        if let Ok(creds) = Credentials::retrieve() {
            let try_login_action = if creds.token_expired() {
                TryLoginAction::Password {
                    username: creds.username,
                    password: creds.password,
                }
            } else {
                TryLoginAction::Token {
                    username: creds.username,
                    token: creds.token,
                }
            };
            self.dispatcher
                .dispatch(LoginAction::TryLogin(try_login_action).into());
            true
        } else {
            false
        }
    }

    pub fn clear_saved_credentials(&self) {
        let _ = Credentials::logout();
    }

    pub fn save_token(&self, token: String, token_expiry_time: SystemTime) {
        if let Ok(mut credentials) = Credentials::retrieve() {
            credentials.token = token;
            credentials.token_expiry_time = Some(token_expiry_time);
            self.save_for_autologin(credentials);
        }
    }

    pub fn save_for_autologin(&self, credentials: Credentials) {
        if credentials.save().is_err() {
            self.dispatcher
                .dispatch(AppAction::ShowNotification(gettext(
                    // translators: This notification shows up right after login if the password could not be stored in the keyring (that is, GNOME's keyring aka seahorse, or any other libsecret compliant secret store).
                    "Could not save password. Make sure the session keyring is unlocked.",
                )));
        }
    }

    pub fn login(&self, username: String, password: String) {
        self.dispatcher
            .dispatch(LoginAction::TryLogin(TryLoginAction::Password { username, password }).into())
    }
}
