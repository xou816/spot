use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::{Component, EventListener, PlaylistFactory};
use crate::app::{ActionDispatcher, AppEvent, AppModel};

#[derive(Clone, Gladis)]
struct NowPlayingWidget {
    root: gtk::Widget,
    listbox: gtk::ListBox,
}

impl NowPlayingWidget {
    fn new() -> Self {
        Self::from_resource(resource!("/components/now_playing.ui")).unwrap()
    }
}

pub struct NowPlayingFactory {
    // will change
    playlist_factory: PlaylistFactory,
}

impl NowPlayingFactory {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        let playlist_factory = PlaylistFactory::new(app_model, dispatcher);
        Self { playlist_factory }
    }

    pub fn make_now_playing(&self) -> NowPlaying {
        NowPlaying::new(self.playlist_factory.clone())
    }
}

pub struct NowPlaying {
    widget: NowPlayingWidget,
    children: Vec<Box<dyn EventListener>>,
}

impl NowPlaying {
    pub fn new(playlist_factory: PlaylistFactory) -> Self {
        let widget = NowPlayingWidget::new();
        let playlist = playlist_factory.make_current_playlist(widget.listbox.clone());
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
