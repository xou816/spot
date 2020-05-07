use gtk::prelude::*;
use gtk::EntryExt;
use gio::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

use crate::app::AppAction;
use crate::app::components::{Component, Dispatcher};

pub struct Login {
    dialog: gtk::Dialog,
    username: gtk::Entry,
    password: gtk::Entry,
    login_btn: gtk::Button,
    dispatcher: Dispatcher
}

impl Login {

    pub fn new(builder: &gtk::Builder, dispatcher: Dispatcher) -> Self {

        let dialog: gtk::Dialog = builder.get_object("login").unwrap();
        let username: gtk::Entry = builder.get_object("username").unwrap();
        let password: gtk::Entry = builder.get_object("password").unwrap();
        let login_btn: gtk::Button = builder.get_object("login_btn").unwrap();

        let username_clone = username.clone();
        let password_clone = password.clone();
        let dialog_clone = dialog.clone();
        let dispatcher_clone = dispatcher.clone();
        login_btn.connect_clicked(move |_| {
            let username = username_clone.get_text().unwrap().as_str().to_string();
            let password = password_clone.get_text().unwrap().as_str().to_string();
            dispatcher_clone.send(AppAction::TryLogin(username, password)).unwrap();
            dialog_clone.hide();
        });

        Self { dialog, username: username.clone(), password: password.clone(), login_btn, dispatcher }
    }
}

impl Component for Login {

    fn handle(&self, action: AppAction) {
        if let AppAction::ShowLogin = action {
            self.dialog.run();
        }
    }
}
