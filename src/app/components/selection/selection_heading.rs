use gettextrs::*;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::state::{SelectionContext, SelectionEvent};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, BrowserEvent};

pub struct SelectionHeadingModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SelectionHeadingModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn set_selection_mode(&self, active: bool) {
        self.dispatcher
            .dispatch(AppAction::ChangeSelectionMode(active))
    }

    fn selected_count(&self) -> usize {
        self.app_model.get_state().selection.count()
    }

    fn should_change_context(&self) -> bool {
        let state = self.app_model.get_state();
        state.selection.context != SelectionContext::Global
            && state.recommended_context() != state.selection.context
    }
}

pub struct SelectionHeading {
    model: Rc<SelectionHeadingModel>,
    headerbar: libhandy::HeaderBar,
    selection_toggle: gtk::ToggleButton,
    selection_label: gtk::Label,
}

impl SelectionHeading {
    pub fn new(
        model: SelectionHeadingModel,
        headerbar: libhandy::HeaderBar,
        selection_toggle: gtk::ToggleButton,
        selection_label: gtk::Label,
    ) -> Self {
        let model = Rc::new(model);

        selection_toggle.connect_clicked(clone!(@weak model => move |t| {
            model.set_selection_mode(t.is_active());
        }));

        Self {
            model,
            headerbar,
            selection_toggle,
            selection_label,
        }
    }

    fn set_selection_active(&self, active: bool) {
        let context = self.headerbar.style_context();
        if active {
            self.selection_label.show();
            context.add_class("selection-mode");
        } else {
            self.selection_label.hide();
            self.selection_label.set_label(&gettext("No song selected"));
            context.remove_class("selection-mode");
        }
        if self.selection_toggle.is_active() != active {
            self.selection_toggle.set_active(active);
        }
    }

    fn update_selection_count(&self) {
        let count = self.model.selected_count();
        self.selection_label.set_label(&format!(
            "{} {}",
            count,
            // translators: This is part of a larger text that says "<n> songs selected" when in selection mode. This text should be as short as possible.
            ngettext("song selected", "songs selected", count as u32),
        ));
    }
}

impl EventListener for SelectionHeading {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(active)) => {
                self.set_selection_active(*active);
            }
            AppEvent::SelectionEvent(SelectionEvent::SelectionChanged) => {
                self.update_selection_count();
            }
            // TODO: better navigation management so that we can reliably toggle some selection based on navigation
            AppEvent::BrowserEvent(BrowserEvent::NavigationPushed(_))
            | AppEvent::BrowserEvent(BrowserEvent::NavigationPoppedTo(_))
            | AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                if self.model.should_change_context() {
                    self.model.set_selection_mode(false);
                }
            }
            _ => {}
        }
    }
}
