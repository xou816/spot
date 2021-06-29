use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::utils::Debouncer;
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
    debouncer: Debouncer,
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
            debouncer: Debouncer::new(),
        }
    }
    fn show(&self, content: &str) {
        self.debouncer.debounce(
            4000,
            clone!(@weak self.model as model => move || {
                model.close();
            }),
        );
        self.content.set_text(content);
        self.root.show();
        self.root.style_context().add_class("notification--shown")
    }

    fn hide(&self) {
        self.root.hide();
        self.root
            .style_context()
            .remove_class("notification--shown")
    }
}

impl EventListener for Notification {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::NotificationShown(content) => self.show(&content),
            AppEvent::NotificationHidden => self.hide(),
            // _ if cfg!(debug_assertions) => self.show(&format!("{:?}", event)),
            _ => {}
        }
    }
}
