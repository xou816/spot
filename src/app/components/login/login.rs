use gio::ApplicationExt;
use gtk::prelude::*;
use gtk::{EntryExt, GtkWindowExt, WidgetExt};
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::AppEvent;

use super::LoginModel;

pub struct Login {
    dialog: gtk::Dialog,
    parent: gtk::Window,
    model: Rc<LoginModel>,
}

impl Login {
    pub fn new(
        parent: gtk::Window,
        dialog: gtk::Dialog,
        username: gtk::Entry,
        password: gtk::Entry,
        login_btn: gtk::Button,
        model: LoginModel,
    ) -> Self {
        let model = Rc::new(model);
        login_btn.connect_clicked(
            clone!(@weak username, @weak password,  @weak model => move |_| {
                let username = username.get_text().as_str().to_string();
                let password = password.get_text().as_str().to_string();
                model.login(username, password);
            }),
        );

        dialog.connect_delete_event(
            clone!(@weak parent => @default-return gtk::Inhibit(false), move |_, _| {
                if let Some(app) = parent.get_application().as_ref() {
                    app.quit();
                }
                gtk::Inhibit(true)
            }),
        );

        Self {
            dialog,
            parent,
            model,
        }
    }

    fn show_self_if_needed(&self) {
        if self.model.try_autologin() {
            self.dialog.close();
            return;
        }
        self.show_self();
    }

    fn show_self(&self) {
        self.dialog.set_transient_for(Some(&self.parent));
        self.dialog.set_modal(true);
        self.dialog.show_all();
    }

    fn hide(&self) {
        self.dialog.hide();
    }
}

impl EventListener for Login {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::LoginCompleted => {
                self.hide();
            }
            AppEvent::Started => {
                self.show_self_if_needed();
            }
            AppEvent::LogoutCompleted => {
                self.show_self();
            }
            _ => {}
        }
    }
}
