use gtk::prelude::*;
use super::{Component, EventListener};
use crate::app::connect::PlaylistFactory;
use crate::app::AppEvent;


pub struct Details {
    root: gtk::Widget,
    children: Vec<Box<dyn EventListener>>
}

impl Details {

    pub fn new(playlist_factory: &PlaylistFactory) -> Self {

        let listbox = gtk::ListBoxBuilder::new()
            .selection_mode(gtk::SelectionMode::None)
            .build();

        let scroll_window = gtk::ScrolledWindowBuilder::new()
            .child(&listbox)
            .build();

        let playlist = Box::new(playlist_factory.make_custom_playlist(listbox.clone()));

        Self { root: scroll_window.upcast(), children: vec![playlist] }
    }

    fn broadcast_event(&self, event: &AppEvent) {
        for child in self.children.iter() {
            child.on_event(event);
        }
    }
}

impl Component for Details {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.root
    }
}


impl EventListener for Details {

    fn on_event(&self, event: &AppEvent) {
        self.broadcast_event(event);
    }
}
