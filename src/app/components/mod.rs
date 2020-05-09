use glib::Sender;

use crate::app::{AppAction};

pub mod playback;
pub use playback::{Playback, PlaybackState};

pub mod playlist;
pub use playlist::{Playlist, PlaylistState};

pub mod login;
pub use login::{Login};

mod gtypes;

pub trait Component {
    fn handle(&self, action: AppAction);
}

pub struct Dispatcher {
    sender: Sender<AppAction>
}

impl Dispatcher {
    pub fn new(sender: Sender<AppAction>) -> Self {
        Self { sender }
    }

    pub fn send(&self, action: AppAction) -> Result<(), ()> {
        self.sender.send(action).map_err(|_| ())
    }
}

impl Clone for Dispatcher {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}
