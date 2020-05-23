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
