use gtk::{EntryExt, WidgetExt};

use crate::app::credentials;
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
            self.dispatcher
                .dispatch(AppAction::TryLogin(creds.username, creds.password));
            true
        } else {
            false
        }
    }

    pub fn save_for_autologin(&self, credentials: credentials::Credentials) {
        if credentials::save_credentials(credentials).is_err() {
            self.dispatcher.dispatch(AppAction::ShowNotification(
                "Could not save password.".to_string(),
            ));
        }
    }

    pub fn login(&self, u: String, p: String) {
        self.dispatcher.dispatch(AppAction::TryLogin(u, p));
    }

    pub fn submit_login_form(&self, username: gtk::Entry, password: gtk::Entry) {
        let username_text = username.get_text().as_str().to_string();
        let password_text = password.get_text().as_str().to_string();
        if username_text.is_empty() {
            username.grab_focus();
        } else if password_text.is_empty() {
            password.grab_focus();
        } else {
            self.login(username_text, password_text);
        }
    }
}
