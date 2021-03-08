use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel};

pub struct SelectionEditorModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SelectionEditorModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn toggle_selection(&self) {
        self.dispatcher.dispatch(AppAction::ToggleSelectionMode)
    }

    fn selected_count(&self) -> usize {
        self.app_model
            .get_state()
            .selection
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

pub struct SelectionEditor {
    model: Rc<SelectionEditorModel>,
    headerbar: libhandy::HeaderBar,
    selection_button: gtk::ToggleButton,
    selection_label: gtk::Label,
}

impl SelectionEditor {
    pub fn new(
        model: SelectionEditorModel,
        headerbar: libhandy::HeaderBar,
        selection_toggle: gtk::ToggleButton,
        selection_button: gtk::ToggleButton,
        selection_label: gtk::Label,
    ) -> Self {
        let model = Rc::new(model);
        selection_toggle.connect_clicked(clone!(@weak model => move |_| {
            model.toggle_selection();
        }));

        Self {
            model,
            headerbar,
            selection_button,
            selection_label,
        }
    }

    fn set_selection_active(&self, active: bool) {
        let context = self.headerbar.get_style_context();
        if active {
            self.selection_button.show();
            context.add_class("selection-mode");
        } else {
            self.selection_button.hide();
            self.selection_label.set_label("No songs selected");
            context.remove_class("selection-mode");
        }
    }

    fn update_selection_count(&self) {
        self.selection_label
            .set_label(&format!("{} songs selected", self.model.selected_count()));
    }
}

impl EventListener for SelectionEditor {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::SelectionModeChanged(active) => {
                self.set_selection_active(*active);
            }
            AppEvent::Selected(_) | AppEvent::Deselected(_) => {
                self.update_selection_count();
            }
            _ => {}
        }
    }
}
