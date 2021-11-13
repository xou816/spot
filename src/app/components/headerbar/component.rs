use std::rc::Rc;

use glib::Cast;

use crate::app::{
    components::{Component, EventListener, ListenerComponent},
    state::SelectionEvent,
    ActionDispatcher, AppAction, AppEvent, AppModel, BrowserAction, BrowserEvent,
};

use super::widget::HeaderBarWidget;

pub trait StandardScreenModel {
    fn title(&self) -> Option<&str>;
    fn go_back(&self);
    fn can_go_back(&self) -> bool;
    fn can_select(&self) -> bool;
    fn start_selection(&self);
    fn select_all(&self);
    fn cancel_selection(&self);
    fn selected_count(&self) -> usize;
}

pub struct DefaultScreenModel {
    title: Option<String>,
    can_select: bool,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl DefaultScreenModel {
    pub fn new(
        title: Option<String>,
        can_select: bool,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Self {
        Self {
            title,
            can_select,
            app_model,
            dispatcher,
        }
    }
}

impl StandardScreenModel for DefaultScreenModel {
    fn title(&self) -> Option<&str> {
        Some(&self.title.as_ref()?)
    }

    fn go_back(&self) {
        self.dispatcher
            .dispatch(BrowserAction::NavigationPop.into())
    }

    fn can_go_back(&self) -> bool {
        self.app_model.get_state().browser.can_pop()
    }

    fn can_select(&self) -> bool {
        self.can_select
    }

    fn start_selection(&self) {
        self.dispatcher
            .dispatch(AppAction::ChangeSelectionMode(true))
    }

    fn select_all(&self) {}

    fn cancel_selection(&self) {
        self.dispatcher
            .dispatch(AppAction::ChangeSelectionMode(false))
    }

    fn selected_count(&self) -> usize {
        self.app_model.get_state().selection.count()
    }
}

pub struct StandardScreen<Model: StandardScreenModel> {
    widget: HeaderBarWidget,
    model: Rc<Model>,
    children: Vec<Box<dyn EventListener>>,
}

impl<Model> StandardScreen<Model>
where
    Model: StandardScreenModel + 'static,
{
    pub fn new(wrapped: impl ListenerComponent + 'static, model: Rc<Model>) -> Self {
        let widget = HeaderBarWidget::new();

        widget.add(wrapped.get_root_widget());

        widget.connect_selection_start(clone!(@weak model => move || model.start_selection()));
        widget.connect_selection_cancel(clone!(@weak model => move || model.cancel_selection()));
        widget.connect_go_back(clone!(@weak model => move || model.go_back()));

        widget.set_title(model.title());
        widget.set_selection_possible(model.can_select());
        widget.set_can_go_back(model.can_go_back());

        Self {
            widget,
            model,
            children: vec![Box::new(wrapped)],
        }
    }
}

impl<Model> Component for StandardScreen<Model>
where
    Model: StandardScreenModel + 'static,
{
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl<Model> EventListener for StandardScreen<Model>
where
    Model: StandardScreenModel + 'static,
{
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(active)) => {
                self.widget.set_selection_active(*active);
            }
            AppEvent::SelectionEvent(SelectionEvent::SelectionChanged) => {
                self.widget.set_selection_count(self.model.selected_count());
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPushed(_))
            | AppEvent::BrowserEvent(BrowserEvent::NavigationPoppedTo(_))
            | AppEvent::BrowserEvent(BrowserEvent::NavigationPopped)
            | AppEvent::BrowserEvent(BrowserEvent::NavigationHidden(_)) => {
                self.model.cancel_selection();
                self.widget.set_can_go_back(self.model.can_go_back())
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
