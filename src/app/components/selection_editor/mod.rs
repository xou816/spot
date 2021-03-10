use gio::{ActionMapExt, SimpleAction, SimpleActionGroup};
use gtk::prelude::*;
use gtk::ButtonExt;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::state::{SelectionAction, SelectionEvent};
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

    fn set_selection_mode(&self, active: bool) {
        self.dispatcher
            .dispatch(SelectionAction::ChangeSelectionMode(active).into())
    }

    fn selected_count(&self) -> usize {
        self.app_model.get_state().selection.count()
    }

    fn all_selected_from_queue(&self) -> bool {
        self.app_model.get_state().selection_is_from_queue()
    }

    fn make_actions(&self) -> SimpleActionGroup {
        let group = SimpleActionGroup::new();

        let queue_selection = SimpleAction::new("queue", None);
        let dispatcher = self.dispatcher.box_clone();
        queue_selection.connect_activate(move |_, _| {
            dispatcher.dispatch(AppAction::QueueSelection);
        });
        group.add_action(&queue_selection);

        let dequeue_selection = SimpleAction::new("dequeue", None);
        let dispatcher = self.dispatcher.box_clone();
        dequeue_selection.connect_activate(move |_, _| {
            dispatcher.dispatch(AppAction::DequeueSelection);
        });
        group.add_action(&dequeue_selection);

        group
    }
}

pub struct SelectionEditor {
    model: Rc<SelectionEditorModel>,
    headerbar: libhandy::HeaderBar,
    selection_toggle: gtk::ToggleButton,
    selection_button: gtk::MenuButton,
    selection_label: gtk::Label,
}

impl SelectionEditor {
    pub fn new(
        model: SelectionEditorModel,
        headerbar: libhandy::HeaderBar,
        selection_toggle: gtk::ToggleButton,
        selection_button: gtk::MenuButton,
        selection_label: gtk::Label,
    ) -> Self {
        let model = Rc::new(model);
        selection_toggle.connect_clicked(clone!(@weak model => move |t| {
            model.set_selection_mode(t.get_active());
        }));

        selection_button.insert_action_group("selection", Some(&model.make_actions()));

        Self {
            model,
            headerbar,
            selection_toggle,
            selection_button,
            selection_label,
        }
    }

    fn set_selection_active(&self, active: bool) {
        let context = self.headerbar.get_style_context();
        if active {
            self.selection_button.set_sensitive(false);
            self.selection_button.set_relief(gtk::ReliefStyle::None);
            self.selection_button.show();
            context.add_class("selection-mode");
        } else {
            self.selection_button.hide();
            self.selection_label.set_label("No songs selected");
            context.remove_class("selection-mode");
        }
        if self.selection_toggle.get_active() != active {
            self.selection_toggle.set_active(active);
        }
    }

    fn update_selection_count(&self) {
        let count = self.model.selected_count();
        self.selection_button.set_relief(if count > 0 {
            gtk::ReliefStyle::Normal
        } else {
            gtk::ReliefStyle::None
        });
        self.selection_button.set_sensitive(count > 0);
        self.selection_label
            .set_label(&format!("{} songs selected", count));
    }

    fn update_selection_actions(&self) {
        if self.model.selected_count() == 0 {
            return;
        }

        let menu = gio::Menu::new();
        if self.model.all_selected_from_queue() {
            menu.append(Some("Dequeue selected"), Some("selection.dequeue"));
        } else {
            menu.append(Some("Queue selected"), Some("selection.queue"));
        }
        self.selection_button.set_menu_model(Some(&menu));
    }
}

impl EventListener for SelectionEditor {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(active)) => {
                self.set_selection_active(*active);
            }
            AppEvent::SelectionEvent(SelectionEvent::Selected(_))
            | AppEvent::SelectionEvent(SelectionEvent::Deselected(_)) => {
                self.update_selection_count();
                self.update_selection_actions();
            }
            _ => {}
        }
    }
}
