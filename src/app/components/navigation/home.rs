use gettextrs::*;
use gtk::prelude::*;
use gtk::{ListBoxExt, StackExt, StackSidebarExt};

use crate::app::components::{Component, EventListener, ScreenFactory};
use crate::app::AppEvent;

fn find_listbox_descendant(w: &gtk::Widget) -> Option<gtk::ListBox> {
    match w.clone().downcast::<gtk::ListBox>() {
        Ok(listbox) => Some(listbox),
        Err(widget) => {
            let next = widget.downcast::<gtk::Bin>().ok()?.get_child()?;
            find_listbox_descendant(&next)
        }
    }
}

pub struct HomePane {
    stack: gtk::Stack,
    stack_sidebar: gtk::StackSidebar,
    components: Vec<Box<dyn EventListener>>,
}

impl HomePane {
    pub fn new(stack_sidebar: gtk::StackSidebar, screen_factory: &ScreenFactory) -> Self {
        let library = screen_factory.make_library();
        let saved_playlists = screen_factory.make_saved_playlists();
        let now_playing = screen_factory.make_now_playing();

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        stack.add_titled(library.get_root_widget(), "library", &gettext("Library"));
        stack.add_titled(
            saved_playlists.get_root_widget(),
            "saved_playlists",
            &gettext("Playlists"),
        );
        stack.add_titled(
            now_playing.get_root_widget(),
            "now_playing",
            &gettext("Now playing"),
        );

        stack_sidebar.set_stack(&stack);

        Self {
            stack,
            stack_sidebar,
            components: vec![
                Box::new(library),
                Box::new(saved_playlists),
                Box::new(now_playing),
            ],
        }
    }

    pub fn connect_navigated<F: Fn() + 'static>(&self, f: F) {
        // stack sidebar wraps a listbox with a scroll window, so i'm cheating a bit there to get the listbox ;)
        if let Some(listbox) = find_listbox_descendant(self.stack_sidebar.upcast_ref()) {
            listbox.connect_row_activated(move |_, _| {
                f();
            });
        }
    }
}

impl Component for HomePane {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.stack.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.components)
    }
}

impl EventListener for HomePane {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::NowPlayingShown = event {
            self.stack.set_visible_child_name("now_playing");
        }
        self.broadcast_event(event);
    }
}
