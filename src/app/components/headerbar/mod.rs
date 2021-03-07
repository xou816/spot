use gtk::prelude::*;
use libhandy::HeaderBarExt;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::{ActionDispatcher, AppAction, AppEvent};

pub struct HeaderBarModel {
    dispatcher: Box<dyn ActionDispatcher>,
}

impl HeaderBarModel {
    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }

    fn toggle_selection(&self) {
        self.dispatcher.dispatch(AppAction::ToggleSelectionMode)
    }
}

pub struct HeaderBar {
    headerbar: libhandy::HeaderBar,
}

impl HeaderBar {
    pub fn new(
        model: HeaderBarModel,
        headerbar: libhandy::HeaderBar,
        selection_toggle: gtk::ToggleButton,
    ) -> Self {
        let model = Rc::new(model);
        selection_toggle.connect_clicked(move |_| {
            model.toggle_selection();
        });

        Self { headerbar }
    }

    pub fn set_selection_active(&self, active: bool) {
        let context = self.headerbar.get_style_context();
        if active {
            self.headerbar.set_title(Some("Select songs"));
            context.add_class("selection-mode");
        } else {
            self.headerbar.set_title(Some("Spot"));
            context.remove_class("selection-mode");
        }
    }
}

impl EventListener for HeaderBar {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::SelectionModeChanged(active) => {
                self.set_selection_active(*active);
            }
            _ => {}
        }
    }
}
