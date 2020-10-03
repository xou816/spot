use std::rc::Rc;
use std::cell::RefCell;

use crate::app::{AppModel, AppAction};
use crate::app::components::{LoginModel};
use crate::app::credentials;

pub struct LoginModelImpl(pub Rc<RefCell<AppModel>>);

impl LoginModelImpl {
    fn dispatch(&self, action: AppAction) {
        self.0.borrow().dispatch(action);
    }
}

impl LoginModel for LoginModelImpl {

    fn try_autologin(&self) -> bool {
        if let Ok(creds) = credentials::try_retrieve_credentials() {
            self.dispatch(AppAction::TryLogin(creds.username, creds.password));
            true
        } else {
            false
        }
    }

    fn login(&self, u: String, p: String) {
        self.dispatch(AppAction::TryLogin(u, p));
    }
}
