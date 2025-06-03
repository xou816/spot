use crate::app::state::{LoginAction, TryLoginAction};
use crate::app::ActionDispatcher;

pub struct LoginModel {
    dispatcher: Box<dyn ActionDispatcher>,
}

impl LoginModel {
    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }

    pub fn try_autologin(&self) {
        self.dispatcher
            .dispatch(LoginAction::TryLogin(TryLoginAction::Restore).into());
    }

    pub fn login_with_spotify(&self) {
        self.dispatcher
            .dispatch(LoginAction::TryLogin(TryLoginAction::InitLogin).into())
    }
}
