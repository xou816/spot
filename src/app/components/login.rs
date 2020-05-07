use gtk::prelude::*;
use gio::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

use crate::app::AppAction;
use crate::app::components::{Component, Dispatcher};

pub struct Login {
    dialog: gtk::MessageDialog
}

impl Login {

    pub fn new(builder: &gtk::Builder) -> Self {
        Self { dialog: builder.get_object("login").unwrap() }
    }
}

impl Component for Login {

    fn handle(&self, action: AppAction) {
        if let AppAction::ShowLogin = action {
            self.dialog.run();
        }
    }
}
