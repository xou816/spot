use gtk::prelude::*;
use gtk::EntryExt;
use std::rc::Rc;
use std::cell::RefCell;

use crate::app::{AppAction};
use crate::app::components::{Component};

pub struct Login {
    dialog: gtk::Dialog,
    parent: gtk::Window,
    model: Rc<RefCell<dyn LoginModel>>
}

pub trait LoginModel {
    fn try_autologin(&self) -> bool;
    fn login(&self, u: String, p: String);
}

impl Login {

    pub fn new(builder: &gtk::Builder, model: Rc<RefCell<dyn LoginModel>>) -> Self {

        let parent: gtk::Window = builder.get_object("window").unwrap();
        let dialog: gtk::Dialog = builder.get_object("login").unwrap();
        let username: gtk::Entry = builder.get_object("username").unwrap();
        let password: gtk::Entry = builder.get_object("password").unwrap();
        let login_btn: gtk::Button = builder.get_object("login_btn").unwrap();

        let weak_model = Rc::downgrade(&model);
        login_btn.connect_clicked(move |_| {
            let username = username.get_text().unwrap().as_str().to_string();
            let password = password.get_text().unwrap().as_str().to_string();
            weak_model.upgrade().map(|m| m.borrow().login(username, password));
        });

        dialog.connect_delete_event(|_, _| {
            gtk::Inhibit(true)
        });

        Self { dialog, parent, model }
    }

    fn show_self_if_needed(&self) {
        if self.model.borrow().try_autologin() {
            self.dialog.destroy();
            return
        }
        self.dialog.set_transient_for(Some(&self.parent));
        self.dialog.set_modal(true);
        self.dialog.show_all();
    }

    fn hide(&self) {
        self.dialog.hide();
    }
}

impl Component for Login {

    fn handle(&self, action: &AppAction) {
        if let AppAction::StartLogin = action {
            self.show_self_if_needed();
        } else if let AppAction::LoginSuccess(_) = action {
            self.hide();
        }
    }
}
