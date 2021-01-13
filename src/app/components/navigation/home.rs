use gtk::prelude::*;
use gtk::{StackExt, StackSidebarExt};

use crate::app::components::{Browser, Component, EventListener};
use crate::app::AppEvent;

pub struct HomeComponent {
    stack: gtk::Stack,
    components: Vec<Box<dyn EventListener>>,
}

impl HomeComponent {
    pub fn new(stack_sidebar: gtk::StackSidebar, library: Browser) -> Self {
        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        stack.add_titled(library.get_root_widget(), "library", "Library");

        stack_sidebar.set_stack(&stack);

        Self {
            stack,
            components: vec![Box::new(library)],
        }
    }

    pub fn connect_navigated<F: Fn() + 'static>(&self, f: F) {
        self.stack.connect_property_visible_child_notify(move |_| {
            f();
        });
    }
}

impl Component for HomeComponent {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.stack.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.components)
    }
}

impl EventListener for HomeComponent {
    fn on_event(&mut self, event: &AppEvent) {
        self.broadcast_event(event);
    }
}
