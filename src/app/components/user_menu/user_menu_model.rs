use crate::app::credentials;
use crate::app::{ActionDispatcher, AppAction, AppModel};
use std::ops::Deref;
use std::rc::Rc;

pub struct UserMenuModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl UserMenuModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn username(&self) -> Option<impl Deref<Target = String> + '_> {
        self.app_model.map_state_opt(|s| s.user.as_ref())
    }

    pub fn logout(&self) {
        if credentials::logout().is_ok() {
            self.dispatcher.dispatch(AppAction::Logout);
        }
    }
}
