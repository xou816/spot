use gettextrs::*;

use crate::app::credentials;
use crate::app::{state::LoginAction, ActionDispatcher, AppAction};

pub struct LoginModel {
    dispatcher: Box<dyn ActionDispatcher>,
}

impl LoginModel {
    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }

    pub fn try_autologin(&self) -> bool {
        if let Ok(creds) = credentials::try_retrieve_credentials() {
            self.dispatcher
                .dispatch(LoginAction::TryLogin(creds.username, creds.password).into());
            true
        } else {
            false
        }
    }

    pub fn save_for_autologin(&self, credentials: credentials::Credentials) {
        if credentials::save_credentials(credentials).is_err() {
            self.dispatcher
                .dispatch(AppAction::ShowNotification(gettext(
                    "Could not save password. Make sure the session keyring is unlocked.",
                )));
        }
    }

    pub fn login(&self, u: String, p: String) {
        self.dispatcher.dispatch(LoginAction::TryLogin(u, p).into());
    }
}
