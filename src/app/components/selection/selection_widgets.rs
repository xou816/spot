use gettextrs::*;
use gio::{ActionMapExt, SimpleAction, SimpleActionGroup};
use gtk::prelude::*;
use gtk::ButtonExt;
use gtk::OverlayExt;
use std::rc::Rc;

use crate::app::components::{labels, Component, EventListener, ListenerComponent};
use crate::app::state::{SelectionEvent, SelectionState};
use crate::app::AppEvent;

use super::selection_tools::*;

impl SimpleSelectionTool {
    fn icon_name(&self) -> &'static str {
        match self {
            Self::MoveDown => "go-down-symbolic",
            Self::MoveUp => "go-up-symbolic",
            Self::RemoveFromQueue => "list-remove-symbolic",
            Self::SelectAll => "checkbox-checked-symbolic",
        }
    }
}

trait SelectionWidget {
    fn update_for_state(&self, selection: &SelectionState);
}

struct AddSelectionButton {
    button: gtk::MenuButton,
}

impl AddSelectionButton {
    fn new(tools: Vec<AddSelectionTool>, model: &Rc<impl SelectionToolsModel + 'static>) -> Self {
        let image = gtk::ImageBuilder::new()
            .icon_name("list-add-symbolic")
            .icon_size(gtk::IconSize::LargeToolbar.into())
            .build();
        let button = gtk::MenuButtonBuilder::new()
            .visible(true)
            .image(&image)
            .build();
        button.get_style_context().add_class("osd");

        let action_group = SimpleActionGroup::new();
        button.insert_action_group("add_to", Some(&action_group));

        let menu = gio::Menu::new();
        for tool in tools {
            match tool {
                AddSelectionTool::AddToPlaylist(desc) => {
                    let action_name = format!("playlist_{}", &desc.id);
                    let title = desc.title.clone();
                    action_group.add_action(&{
                        let queue = SimpleAction::new(&action_name, None);
                        queue.connect_activate(clone!(@weak model => move |_, _| {
                            let selection = model.enabled_selection();
                            if let Some(selection) = selection {
                                model.handle_tool_activated(&*selection, &SelectionTool::Add(AddSelectionTool::AddToPlaylist(desc.clone())));
                            }
                        }));
                        queue
                    });
                    menu.append(
                        // translators: This is part of a larger text that says "Add to <playlist name>". This text should be as short as possible.
                        Some(&format!("{} {}", gettext("Add to"), title)),
                        Some(&format!("add_to.{}", action_name)),
                    );
                }
                AddSelectionTool::AddToQueue => {
                    action_group.add_action(&{
                        let queue = SimpleAction::new("queue", None);
                        queue.connect_activate(clone!(@weak model => move |_, _| {
                            let selection = model.enabled_selection();
                            if let Some(selection) = selection {
                                model.handle_tool_activated(&*selection, &SelectionTool::Add(AddSelectionTool::AddToQueue));
                            }
                        }));
                        queue
                    });
                    menu.append(Some(&labels::ADD_TO_QUEUE), Some("add_to.queue"));
                }
            }
        }

        let popover = gtk::Popover::from_model(Some(&button), &menu);
        popover.get_style_context().add_class("osd");
        button.set_popover(Some(&popover));
        gtk::MenuButtonExt::set_direction(&button, gtk::ArrowType::Up);

        Self { button }
    }
}

impl SelectionWidget for AddSelectionButton {
    fn update_for_state(&self, selection: &SelectionState) {
        self.button.set_sensitive(selection.count() > 0);
    }
}

struct SelectionButton {
    button: gtk::Button,
    tool: SimpleSelectionTool,
}

impl SelectionButton {
    fn new(tool: SimpleSelectionTool, model: &Rc<impl SelectionToolsModel + 'static>) -> Self {
        let image = gtk::ImageBuilder::new()
            .icon_name(tool.icon_name())
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
                model.handle_tool_activated(&*selection, &SelectionTool::Simple(tool));
            }
        }));
        Self { button, tool }
    }
}

impl SelectionWidget for SelectionButton {
    fn update_for_state(&self, selection: &SelectionState) {
        self.button.set_sensitive(match self.tool {
            SimpleSelectionTool::MoveUp | SimpleSelectionTool::MoveDown => selection.count() == 1,
            SimpleSelectionTool::SelectAll => true,
            _ => selection.count() > 0,
        });
    }
}

pub struct SelectionTools<Model> {
    root: gtk::Widget,
    button_box: gtk::Box,
    children: Vec<Box<dyn EventListener>>,
    selection_widgets: Vec<Box<dyn SelectionWidget>>,
    model: Rc<Model>,
}

impl<Model> SelectionTools<Model>
where
    Model: SelectionToolsModel + 'static,
{
    pub fn new(wrapped: impl ListenerComponent + 'static, model: Rc<Model>) -> Self {
        let (root, button_box) = Self::make_widgets(wrapped.get_root_widget());
        let selection_widgets = Self::make_buttons(&button_box, &model);
        Self {
            root,
            button_box,
            children: vec![Box::new(wrapped)],
            selection_widgets,
            model,
        }
    }

    fn update_active_tools(&self) {
        if let Some(selection) = self.model.enabled_selection() {
            for button in self.selection_widgets.iter() {
                button.update_for_state(&*selection);
            }
        }
    }

    fn update_visible_tools(&mut self) {
        self.selection_widgets = vec![];
        self.button_box.foreach(|w| self.button_box.remove(w));

        if self.model.enabled_selection().is_some() {
            self.selection_widgets = Self::make_buttons(&self.button_box, &self.model);
            self.button_box.show();
        } else {
            self.button_box.hide();
        }
    }

    fn make_buttons(button_box: &gtk::Box, model: &Rc<Model>) -> Vec<Box<dyn SelectionWidget>> {
        if let Some(selection) = model.enabled_selection() {
            let all_tools = model.tools_visible(&*selection);
            let mut other_tools: Vec<Box<dyn SelectionWidget>> = all_tools
                .iter()
                .filter_map(|t| match t {
                    SelectionTool::Simple(tool) => Some({
                        let button = SelectionButton::new(*tool, model);
                        button.update_for_state(&*selection);
                        button_box.add(&button.button);
                        Box::new(button) as Box<dyn SelectionWidget>
                    }),
                    _ => None,
                })
                .collect();

            let add_tools: Vec<AddSelectionTool> = all_tools
                .into_iter()
                .filter_map(|t| match t {
                    SelectionTool::Add(tool) => Some(tool),
                    _ => None,
                })
                .collect();
            if !add_tools.is_empty() {
                let button = AddSelectionButton::new(add_tools, model);
                button.update_for_state(&*selection);
                button_box.add(&button.button);
                other_tools.push(Box::new(button));
            }

            other_tools
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
