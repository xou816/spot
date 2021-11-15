use std::rc::Rc;

use glib::Cast;
use gtk::prelude::*;

use crate::app::{
    components::{Component, EventListener, ListenerComponent},
    state::{SelectionContext, SelectionEvent},
    ActionDispatcher, AppAction, AppEvent, AppModel, BrowserAction, BrowserEvent,
};

use super::widget::HeaderBarWidget;

pub trait ScreenModel {
    fn title(&self) -> Option<String>;
    fn title_updated(&self, event: &AppEvent) -> bool;
    fn go_back(&self);
    fn can_go_back(&self) -> bool;
    fn selection_context(&self) -> Option<&SelectionContext>;
    fn can_select_all(&self) -> bool;
    fn start_selection(&self);
    fn select_all(&self);
    fn cancel_selection(&self);
    fn selected_count(&self) -> usize;
}

pub struct DefaultScreenModel {
    title: Option<String>,
    selection_context: Option<SelectionContext>,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl DefaultScreenModel {
    pub fn new(
        title: Option<String>,
        selection_context: Option<SelectionContext>,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Self {
        Self {
            title,
            selection_context,
            app_model,
            dispatcher,
        }
    }
}

impl ScreenModel for DefaultScreenModel {
    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn title_updated(&self, _: &AppEvent) -> bool {
        false
    }

    fn go_back(&self) {
        self.dispatcher
            .dispatch(BrowserAction::NavigationPop.into())
    }

    fn can_go_back(&self) -> bool {
        self.app_model.get_state().browser.can_pop()
    }

    fn selection_context(&self) -> Option<&SelectionContext> {
        self.selection_context.as_ref()
    }

    fn can_select_all(&self) -> bool {
        false
    }

    fn start_selection(&self) {
        if let Some(context) = self.selection_context.as_ref() {
            self.dispatcher
                .dispatch(AppAction::EnableSelection(context.clone()))
        }
    }

    fn select_all(&self) {}

    fn cancel_selection(&self) {
        self.dispatcher.dispatch(AppAction::CancelSelection)
    }

    fn selected_count(&self) -> usize {
        self.app_model.get_state().selection.count()
    }
}

pub trait SimpleScreenModel {
    fn title(&self) -> Option<String>;
    fn title_updated(&self, event: &AppEvent) -> bool;
    fn selection_context(&self) -> Option<&SelectionContext>;
    fn select_all(&self);
}

pub struct SimpleScreenModelWrapper<M> {
    wrapped_model: Rc<M>,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl<M> SimpleScreenModelWrapper<M> {
    pub fn new(
        wrapped_model: Rc<M>,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Self {
        Self {
            wrapped_model,
            app_model,
            dispatcher,
        }
    }
}

impl<M> ScreenModel for SimpleScreenModelWrapper<M>
where
    M: SimpleScreenModel + 'static,
{
    fn title(&self) -> Option<String> {
        self.wrapped_model.title()
    }

    fn title_updated(&self, event: &AppEvent) -> bool {
        self.wrapped_model.title_updated(event)
    }

    fn go_back(&self) {
        self.dispatcher
            .dispatch(BrowserAction::NavigationPop.into())
    }

    fn can_go_back(&self) -> bool {
        self.app_model.get_state().browser.can_pop()
    }

    fn selection_context(&self) -> Option<&SelectionContext> {
        self.wrapped_model.selection_context()
    }

    fn can_select_all(&self) -> bool {
        true
    }

    fn start_selection(&self) {
        if let Some(context) = self.wrapped_model.selection_context() {
            self.dispatcher
                .dispatch(AppAction::EnableSelection(context.clone()));
        }
    }

    fn select_all(&self) {
        self.wrapped_model.select_all()
    }

    fn cancel_selection(&self) {
        self.dispatcher.dispatch(AppAction::CancelSelection)
    }

    fn selected_count(&self) -> usize {
        self.app_model.get_state().selection.count()
    }
}

pub struct StandardScreen<Model: ScreenModel> {
    root: gtk::Widget,
    widget: HeaderBarWidget,
    model: Rc<Model>,
    children: Vec<Box<dyn EventListener>>,
}

impl<Model> StandardScreen<Model>
where
    Model: ScreenModel + 'static,
{
    pub fn new(wrapped: impl ListenerComponent + 'static, model: Rc<Model>) -> Self {
        let widget = HeaderBarWidget::new();

        widget.connect_selection_start(clone!(@weak model => move || model.start_selection()));
        widget.connect_select_all(clone!(@weak model => move || model.select_all()));
        widget.connect_selection_cancel(clone!(@weak model => move || model.cancel_selection()));
        widget.connect_go_back(clone!(@weak model => move || model.go_back()));

        widget.set_title(model.title().as_ref().map(|s| &s[..]));
        widget.set_selection_possible(model.selection_context().is_some());
        widget.set_select_all_possible(model.can_select_all());
        widget.set_can_go_back(model.can_go_back());

        let root = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .build();
        root.append(&widget);
        root.append(wrapped.get_root_widget());

        Self {
            root: root.upcast(),
            widget,
            model,
            children: vec![Box::new(wrapped)],
        }
    }
}

impl<Model> Component for StandardScreen<Model>
where
    Model: ScreenModel + 'static,
{
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.root
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl<Model> EventListener for StandardScreen<Model>
where
    Model: ScreenModel + 'static,
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
                self.widget.set_can_go_back(self.model.can_go_back());
            }
            event if self.model.title_updated(event) => {
                self.widget
                    .set_title(self.model.title().as_ref().map(|s| &s[..]));
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
