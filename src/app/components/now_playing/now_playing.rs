use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::{screen_add_css_provider, Component, EventListener, Playlist};
use crate::app::{state::PlaybackEvent, AppEvent};

use super::NowPlayingModel;

#[derive(Clone, Gladis)]
struct NowPlayingWidget {
    root: gtk::Widget,
    listbox: gtk::ListBox,
}

impl NowPlayingWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/now_playing.css"));
        Self::from_resource(resource!("/components/now_playing.ui")).unwrap()
    }
}

pub struct NowPlaying {
    widget: NowPlayingWidget,
    model: Rc<NowPlayingModel>,
    children: Vec<Box<dyn EventListener>>,
}

impl NowPlaying {
    pub fn new(model: Rc<NowPlayingModel>) -> Self {
        let widget = NowPlayingWidget::new();
        let playlist = Playlist::new(widget.listbox.clone(), model.clone());

        Self {
            widget,
            model,
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
        if let AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) = event {
            self.model.load_more_if_needed();
        }
        self.broadcast_event(event);
    }
}
