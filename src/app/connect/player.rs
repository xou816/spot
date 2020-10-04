use std::rc::Rc;
use std::cell::{Ref, RefCell};

use crate::app::{AppModel, AppState};
use crate::app::components::{PlayerModel};

pub struct PlayerModelImpl(pub Rc<RefCell<AppModel>>);

impl PlayerModelImpl {
    fn state(&self) -> Ref<AppState> {
        Ref::map(self.0.borrow(), |m| &m.state)
    }
}

impl PlayerModel for PlayerModelImpl {
    fn current_song_uri(&self) -> Option<String> {
        self.state().current_song_uri.clone()
    }
}
