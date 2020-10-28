use std::rc::{Rc};
use std::cell::{Ref, RefCell};

use crate::app::{AppModel, AppState, AppAction, ActionDispatcher, SongDescription};
use crate::app::components::{PlaylistModel};

pub struct PlaylistModelImpl {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl PlaylistModelImpl {

    pub fn new(app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher }
    }

    fn state(&self) -> Ref<'_, AppState> {
        Ref::map(self.app_model.borrow(), |m| &m.state)
    }
}


impl PlaylistModel for PlaylistModelImpl {

    fn current_song_uri(&self) -> Option<String> {
        self.state().current_song_uri.clone()
    }

    fn songs(&self) -> Ref<'_, Vec<SongDescription>> {
        Ref::map(self.state(), |s| &s.playlist)
    }

    fn play_song(&self, uri: String) {
        self.dispatcher.dispatch(AppAction::Load(uri));
    }
}

