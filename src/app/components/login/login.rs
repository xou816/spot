use gtk::prelude::*;
use gtk::EntryExt;
use std::rc::Rc;

use crate::app::AppEvent;
use crate::app::components::EventListener;

pub struct Login {
    dialog: gtk::Dialog,
    parent: gtk::Window,
    model: Rc<dyn LoginModel>
}

pub trait LoginModel {
    fn try_autologin(&self) -> bool;
    fn login(&self, u: String, p: String);
}

impl Login {

    pub fn new(
        parent: gtk::Window,
        dialog: gtk::Dialog,
        username: gtk::Entry,
        password: gtk::Entry,
        login_btn: gtk::Button,
        model: Rc<dyn LoginModel>) -> Self {

        let weak_model = Rc::downgrade(&model);
        login_btn.connect_clicked(move |_| {
            let username = username.get_text().unwrap().as_str().to_string();
            let password = password.get_text().unwrap().as_str().to_string();
            weak_model.upgrade().map(|m| m.login(username, password));
        });

        dialog.connect_delete_event(|_, _| {
            gtk::Inhibit(true)
        });

        Self { dialog, parent, model }
    }

    fn show_self_if_needed(&self) {
        if self.model.try_autologin() {
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

impl EventListener for Login {

    fn on_event(&self, event: &AppEvent) {
        match event {
            AppEvent::LoginCompleted => {
                self.hide();
            },
            AppEvent::Started => {
                self.show_self_if_needed();
            },
            _ => {}
        }
    }
}
