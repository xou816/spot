use gtk::prelude::*;
use std::rc::Rc;

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
    model: Rc<NotificationModel>,
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
        let model = Rc::new(model);
        close_btn.connect_clicked(clone!(@weak model => move |_| model.close()));

        Self {
            model,
            root,
            content,
        }
    }
    fn show(&self, content: &str) {
        glib::timeout_add_local(
            4000,
            clone!(@weak self.model as model => @default-return glib::Continue(false), move || {
                model.close();
                glib::Continue(false)
            }),
        );
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
            _ => {}
        }
    }
}
