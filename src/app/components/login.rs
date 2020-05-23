use gtk::prelude::*;
use gtk::EntryExt;

use crate::app::{AppAction, Dispatcher};
use crate::app::components::{Component};

pub struct Login {
    dialog: gtk::Dialog,
    parent: gtk::Window
}

impl Login {

    pub fn new(builder: &gtk::Builder, dispatcher: Dispatcher) -> Self {

        let parent: gtk::Window = builder.get_object("window").unwrap();
        let dialog: gtk::Dialog = builder.get_object("login").unwrap();
        let username: gtk::Entry = builder.get_object("username").unwrap();
        let password: gtk::Entry = builder.get_object("password").unwrap();
        let login_btn: gtk::Button = builder.get_object("login_btn").unwrap();

        login_btn.connect_clicked(move |_| {
            let username = username.get_text().unwrap().as_str().to_string();
            let password = password.get_text().unwrap().as_str().to_string();
            dispatcher.dispatch(AppAction::TryLogin(username, password)).unwrap();
        });

        dialog.connect_delete_event(|_, _| {
            gtk::Inhibit(true)
        });

        Self { dialog, parent }
    }
}

impl Component for Login {

    fn handle(&self, action: &AppAction) {
        if let AppAction::ShowLogin = action {
            self.dialog.set_transient_for(Some(&self.parent));
            self.dialog.set_modal(true);
            self.dialog.show_all();
        } else if let AppAction::LoginSuccess(_) = action {
            self.dialog.hide();
        }
    }
}
