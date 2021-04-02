use gtk::prelude::*;
use gtk::ButtonExt;
use gtk::OverlayExt;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{Component, EventListener, ListenerComponent};
use crate::app::state::{SelectionContext, SelectionEvent, SelectionState};
use crate::app::{AppAction, AppEvent};

#[derive(Debug, Clone, Copy)]
pub enum SelectionTool {
    MoveUp,
    MoveDown,
    Add,
    Remove,
    SelectAll,
}

impl SelectionTool {
    pub fn default_action(&self) -> Option<AppAction> {
        match self {
            Self::MoveDown => Some(AppAction::MoveDownSelection),
            Self::MoveUp => Some(AppAction::MoveUpSelection),
            Self::Remove => Some(AppAction::DequeueSelection),
            Self::Add => Some(AppAction::QueueSelection),
            Self::SelectAll => None,
        }
    }
}

struct SelectionButton {
    button: gtk::Button,
    pub tool: SelectionTool,
}

impl SelectionButton {
    fn new(tool: SelectionTool, model: &Rc<impl SelectionToolsModel + 'static>) -> Self {
        let image = gtk::ImageBuilder::new()
            .icon_name(Self::icon_name(&tool))
            .icon_size(gtk::IconSize::LargeToolbar.into())
            .build();
        let button = gtk::ButtonBuilder::new()
            .visible(true)
            .image(&image)
            .build();
        button.get_style_context().add_class("osd");
        button.connect_clicked(clone!(@weak model => move |_| {
            let selection = model.enabled_selection();
            if let Some(selection) = selection {
                model.handle_tool_activated(&*selection, &tool);
            }
        }));
        Self { button, tool }
    }

    fn icon_name(tool: &SelectionTool) -> &'static str {
        match tool {
            SelectionTool::MoveDown => "go-down-symbolic",
            SelectionTool::MoveUp => "go-up-symbolic",
            SelectionTool::Remove => "list-remove-symbolic",
            SelectionTool::Add => "list-add-symbolic",
            SelectionTool::SelectAll => "checkbox-checked-symbolic",
        }
    }

    fn set_enabled(&self, enabled: bool) {
        self.button.set_sensitive(enabled);
    }
}

pub trait SelectionToolsModel {
    fn tool_enabled(&self, selection: &SelectionState, tool: &SelectionTool) -> bool {
        match tool {
            SelectionTool::MoveUp | SelectionTool::MoveDown => selection.count() == 1,
            SelectionTool::SelectAll => true,
            _ => selection.count() > 0,
        }
    }

    fn tools_for_context(context: &SelectionContext) -> Vec<SelectionTool> {
        match context {
            SelectionContext::Global => vec![SelectionTool::SelectAll, SelectionTool::Add],
            SelectionContext::Queue => vec![
                SelectionTool::SelectAll,
                SelectionTool::MoveDown,
                SelectionTool::MoveUp,
                SelectionTool::Remove,
            ],
        }
    }

    fn tools_visible(&self, selection: &SelectionState) -> Vec<SelectionTool> {
        Self::tools_for_context(&selection.context)
    }

    fn enabled_selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        self.selection().filter(|s| s.is_selection_enabled())
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>>;
    fn handle_tool_activated(&self, selection: &SelectionState, tool: &SelectionTool);
}

pub struct SelectionTools<Model> {
    root: gtk::Widget,
    button_box: gtk::Box,
    children: Vec<Box<dyn EventListener>>,
    buttons: Vec<SelectionButton>,
    model: Rc<Model>,
}

impl<Model> SelectionTools<Model>
where
    Model: SelectionToolsModel + 'static,
{
    pub fn new(wrapped: impl ListenerComponent + 'static, model: Rc<Model>) -> Self {
        let (root, button_box) = Self::make_widgets(wrapped.get_root_widget());
        let buttons = Self::replace_buttons(&button_box, &model);
        Self {
            root,
            button_box,
            children: vec![Box::new(wrapped)],
            buttons,
            model,
        }
    }

    fn update_active_tools(&self) {
        if let Some(selection) = self.model.enabled_selection() {
            for button in self.buttons.iter() {
                button.set_enabled(self.model.tool_enabled(&*selection, &button.tool));
            }
        }
    }

    fn update_visible_tools(&mut self) {
        self.buttons = vec![];
        self.button_box.foreach(|w| self.button_box.remove(w));

        if self.model.enabled_selection().is_some() {
            self.buttons = Self::replace_buttons(&self.button_box, &self.model);
            self.button_box.show();
        } else {
            self.button_box.hide();
        }
    }

    fn replace_buttons(button_box: &gtk::Box, model: &Rc<Model>) -> Vec<SelectionButton> {
        if let Some(selection) = model.enabled_selection() {
            model
                .tools_visible(&*selection)
                .iter()
                .map(|tool| {
                    let button = SelectionButton::new(*tool, model);
                    button.set_enabled(model.tool_enabled(&*selection, &tool));
                    button_box.add(&button.button);
                    button
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn make_button_box() -> gtk::Box {
        let button_box = gtk::BoxBuilder::new()
            .halign(gtk::Align::Center)
            .valign(gtk::Align::End)
            .margin(20)
            .build();
        button_box.get_style_context().add_class("linked");
        button_box
    }

    fn make_widgets(main_child: &gtk::Widget) -> (gtk::Widget, gtk::Box) {
        let root = gtk::OverlayBuilder::new().expand(true).build();
        let button_box = Self::make_button_box();
        root.add(main_child);
        root.add_overlay(&button_box);
        (root.upcast(), button_box)
    }
}

impl<Model> Component for SelectionTools<Model> {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.root
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl<Model> EventListener for SelectionTools<Model>
where
    Model: SelectionToolsModel + 'static,
{
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(_)) => {
                self.update_visible_tools();
            }
            AppEvent::SelectionEvent(SelectionEvent::SelectionChanged) => {
                self.update_active_tools();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
