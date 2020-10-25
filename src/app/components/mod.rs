use crate::app::{AppEvent};

pub mod playback;
pub use playback::{Playback, PlaybackModel};

pub mod playlist;
pub use playlist::{Playlist, PlaylistModel};

pub mod login;
pub use login::{Login, LoginModel};

pub mod player_notifier;
pub use player_notifier::{PlayerNotifier};

pub mod browser;
pub use browser::{Browser, BrowserModel};

mod gtypes;

pub trait Component {
    fn on_event(&self, _: AppEvent) {}
}
