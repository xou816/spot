use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::{screen_add_css_provider, Component, EventListener, Playlist};
use crate::app::AppEvent;

use super::NowPlayingModel;

#[derive(Clone, Gladis)]
struct NowPlayingWidget {
    root: gtk::Widget,
    listbox: gtk::ListBox,
    shuffle: gtk::Button,
}

impl NowPlayingWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/now_playing.css"));
        Self::from_resource(resource!("/components/now_playing.ui")).unwrap()
    }
}

pub struct NowPlaying {
    widget: NowPlayingWidget,
    children: Vec<Box<dyn EventListener>>,
}

impl NowPlaying {
    pub fn new(model: Rc<NowPlayingModel>) -> Self {
        let widget = NowPlayingWidget::new();

        widget
            .shuffle
            .connect_clicked(clone!(@weak model => move |_| {
                model.toggle_shuffle();
            }));

        let playlist = Playlist::new(widget.listbox.clone(), model);

        Self {
            widget,
            children: vec![Box::new(playlist)],
        }
    }
}

impl Component for NowPlaying {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for NowPlaying {
    fn on_event(&mut self, event: &AppEvent) {
        self.broadcast_event(event);
    }
}
