use std::rc::{Rc};
use std::cell::{Ref, RefCell};

use crate::app::{AppModel, AppState, AppAction, SongDescription};
use crate::app::components::{PlaylistModel};

pub struct PlaylistModelImpl(pub Rc<RefCell<AppModel>>);

impl PlaylistModelImpl {
    fn dispatch(&self, action: AppAction) {
        self.0.borrow().dispatch(action);
    }

    fn state(&self) -> Ref<'_, AppState> {
        Ref::map(self.0.borrow(), |m| &m.state)
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
        self.dispatch(AppAction::Load(uri));
    }
}

