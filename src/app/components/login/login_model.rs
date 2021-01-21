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
        if let Err(_) = credentials::save_credentials(credentials) {
            self.dispatcher.dispatch(AppAction::ShowNotification(
                "Could not save password.".to_string(),
            ));
        }
    }

    pub fn login(&self, u: String, p: String) {
        self.dispatcher.dispatch(AppAction::TryLogin(u, p));
    }
}
