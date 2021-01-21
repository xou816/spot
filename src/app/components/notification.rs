use gtk::prelude::*;

use crate::app::components::EventListener;
use crate::app::{ActionDispatcher, AppAction, AppEvent};

pub struct NotificationModel {
    dispatcher: Box<dyn ActionDispatcher>,
}

impl NotificationModel {
    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }

    fn close(&self) {
        self.dispatcher.dispatch(AppAction::HideNotification);
    }
}

pub struct Notification {
    root: gtk::Box,
    content: gtk::Label,
}

impl Notification {
    pub fn new(
        model: NotificationModel,
        root: gtk::Box,
        content: gtk::Label,
        close_btn: gtk::Button,
    ) -> Self {
        close_btn.connect_clicked(move |_| model.close());

        Self { root, content }
    }
    fn show(&self, content: &str) {
        self.content.set_text(content);
        self.root
            .get_style_context()
            .add_class("notification--shown")
    }

    fn hide(&self) {
        self.root
            .get_style_context()
            .remove_class("notification--shown")
    }
}

impl EventListener for Notification {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::NotificationShown(content) => self.show(&content),
            AppEvent::NotificationHidden => self.hide(),
            // AppEvent::Started => self.show("Welcome to Spot!"),
            _ => {}
        }
    }
}
