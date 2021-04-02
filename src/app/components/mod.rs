#[macro_export]
macro_rules! resource {
    ($resource:expr) => {
        concat!("/dev/alextren/Spot", $resource)
    };
}

use gettextrs::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::api::SpotifyApiError;
use crate::app::{state::LoginAction, AppAction, AppEvent};

mod navigation;
pub use navigation::*;

mod playback;
pub use playback::*;

mod playlist;
pub use playlist::*;

mod login;
pub use login::*;

mod player_notifier;
pub use player_notifier::PlayerNotifier;

mod library;
pub use library::*;

mod details;
pub use details::*;

mod search;
pub use search::*;

mod album;
use album::*;

mod artist;
use artist::*;

mod artist_details;
pub use artist_details::*;

mod user_details;
pub use user_details::*;

mod now_playing;
pub use now_playing::*;

mod user_menu;
pub use user_menu::*;

mod notification;
pub use notification::*;

mod saved_playlists;
pub use saved_playlists::*;

mod playlist_details;
pub use playlist_details::*;

mod window;
pub use window::*;

mod selection;
pub use selection::*;

pub mod utils;

pub mod labels;

pub fn handle_error(err: SpotifyApiError) -> Option<AppAction> {
    match err {
        SpotifyApiError::InvalidToken => Some(LoginAction::RefreshToken.into()),
        SpotifyApiError::NoToken => None,
        _ => {
            println!("Error: {:?}", err);
            Some(AppAction::ShowNotification(gettext(
                // translators: This notification is the default message for unhandled errors. Logs refer to console output.
                "An error occured. Check logs for details!",
            )))
        }
    }
}

thread_local!(static CSS_ADDED: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new()));

pub fn screen_add_css_provider(resource: &'static str) {
    CSS_ADDED.with(|set| {
        if set.borrow().contains(resource) {
            return;
        }

        set.borrow_mut().insert(resource);

        let provider = gtk::CssProvider::new();
        provider.load_from_resource(resource);

        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
}

pub trait EventListener {
    fn on_event(&mut self, _: &AppEvent) {}
}

pub trait Component {
    fn get_root_widget(&self) -> &gtk::Widget;

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        None
    }

    fn broadcast_event(&mut self, event: &AppEvent) {
        if let Some(children) = self.get_children() {
            for child in children.iter_mut() {
                child.on_event(event);
            }
        }
    }
}

pub trait ListenerComponent: Component + EventListener {}
impl<T> ListenerComponent for T where T: Component + EventListener {}
