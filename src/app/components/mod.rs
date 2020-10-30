use crate::app::{AppEvent};

pub mod navigation;
pub use navigation::{Navigation, NavigationModel};

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

pub mod details;
pub use details::*;

mod gtypes;

pub trait EventListener {
    fn on_event(&self, _: &AppEvent) {}
}

pub trait Component {
    fn get_root_widget(&self) -> &gtk::Widget;
}

pub trait ListenerComponent: Component + EventListener {}
impl <T> ListenerComponent for T where T: Component + EventListener {}
