use gdk::{keys::constants::Return, EventKey};
use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::credentials::Credentials;
use crate::app::state::{LoginCompletedEvent, LoginEvent};
use crate::app::AppEvent;

use super::LoginModel;

#[derive(Clone, Gladis)]
struct LoginWidget {
    pub window: libhandy::Window,
    username: gtk::Entry,
    password: gtk::Entry,
    close_button: gtk::Button,
    login_button: gtk::Button,
    error_container: gtk::Revealer,
}

impl LoginWidget {
    fn new() -> Self {
        Self::from_resource(resource!("/components/login.ui")).unwrap()
    }
}

pub struct Login {
    parent: gtk::Window,
    window: libhandy::Window,
    error_container: gtk::Revealer,
    model: Rc<LoginModel>,
}

impl Login {
    pub fn new(parent: gtk::Window, model: LoginModel) -> Self {
        let model = Rc::new(model);
        let LoginWidget {
            window,
            username,
            password,
            close_button,
            login_button,
            error_container,
        } = LoginWidget::new();

        login_button.connect_clicked(
            clone!(@weak username, @weak password, @weak error_container,  @weak model => move |_| {
                Self::submit_login_form(username, password, error_container, model);
            }),
        );

        username.connect_key_press_event(
            clone!(@weak username, @weak password, @weak error_container, @weak model => @default-return Inhibit(false), move |_, event | {
                Self::handle_keypress(username, password, error_container, model, event)
            }),
        );

        password.connect_key_press_event(
            clone!(@weak username, @weak password, @weak error_container, @weak model => @default-return Inhibit(false), move |_, event | {
                Self::handle_keypress(username, password, error_container, model, event)
            }),
        );

        close_button.connect_clicked(clone!(@weak parent => move |_| {
            if let Some(app) = parent.application().as_ref() {
                app.quit();
            }
        }));

        Self {
            parent,
            window,
            error_container,
            model,
        }
    }

    fn show_self_if_needed(&self) {
        if self.model.try_autologin() {
            self.window.close();
        } else {
            self.show_self();
        }
    }

    fn show_self(&self) {
        self.window.set_transient_for(Some(&self.parent));
        self.window.set_modal(true);
        self.window.show_all();
    }

    fn hide_and_save_creds(&self, credentials: Credentials) {
        self.window.hide();
        self.model.save_for_autologin(credentials);
    }

    fn handle_keypress(
        username: gtk::Entry,
        password: gtk::Entry,
        error_container: gtk::Revealer,
        model: Rc<LoginModel>,
        event: &EventKey,
    ) -> Inhibit {
        if event.keyval() == Return {
            Login::submit_login_form(username, password, error_container, model);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }

    fn submit_login_form(
        username: gtk::Entry,
        password: gtk::Entry,
        error_container: gtk::Revealer,
        model: Rc<LoginModel>,
    ) {
        error_container.set_reveal_child(false);
        let username_text = username.text().as_str().to_string();
        let password_text = password.text().as_str().to_string();
        if username_text.is_empty() {
            username.grab_focus();
        } else if password_text.is_empty() {
            password.grab_focus();
        } else {
            model.login(username_text, password_text);
        }
    }

    fn reveal_error(&self) {
        self.error_container.set_reveal_child(true);
    }
}

impl EventListener for Login {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::LoginEvent(LoginEvent::LoginCompleted(LoginCompletedEvent::Password(
                creds,
            ))) => {
                self.hide_and_save_creds(creds.clone());
            }
            AppEvent::LoginEvent(LoginEvent::LoginFailed) => {
                self.reveal_error();
            }
            AppEvent::Started => {
                self.show_self_if_needed();
            }
            AppEvent::LoginEvent(LoginEvent::LogoutCompleted) => {
                self.show_self();
            }
            _ => {}
        }
    }
}
