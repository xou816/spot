#[macro_export]
macro_rules! resource {
    ($resource:expr) => {
        concat!("/dev/alextren/Spot", $resource)
    };
}

use gettextrs::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::future::Future;

use crate::api::SpotifyApiError;
use crate::app::{state::LoginAction, ActionDispatcher, AppAction, AppEvent};

mod navigation;
pub use navigation::*;

mod playback;
pub use playback::*;

mod playlist;
pub use playlist::*;

mod login;
pub use login::*;

mod settings;
pub use settings::*;

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

mod device_selector;
pub use device_selector::*;

mod followed_artists;
pub use followed_artists::*;

mod saved_tracks;
pub use saved_tracks::*;

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

mod headerbar;
pub use headerbar::*;

mod scrolling_header;
pub use scrolling_header::*;

pub mod utils;

pub mod labels;

pub mod sidebar;

// without this the builder doesn't seen to know about the custom widgets
pub fn expose_custom_widgets() {
    playback::expose_widgets();
    selection::expose_widgets();
    headerbar::expose_widgets();
    device_selector::expose_widgets();
    playlist_details::expose_widgets();
    scrolling_header::expose_widgets();
}

impl dyn ActionDispatcher {
    fn call_spotify_and_dispatch<F, C>(&self, call: C)
    where
        C: 'static + Send + Clone + FnOnce() -> F,
        F: Send + Future<Output = Result<AppAction, SpotifyApiError>>,
    {
        self.call_spotify_and_dispatch_many(move || async { call().await.map(|a| vec![a]) })
    }

    fn call_spotify_and_dispatch_many<F, C>(&self, call: C)
    where
        C: 'static + Send + Clone + FnOnce() -> F,
        F: Send + Future<Output = Result<Vec<AppAction>, SpotifyApiError>>,
    {
        self.dispatch_many_async(Box::pin(async move {
            let first_call = call.clone();
            let result = first_call().await;
            match result {
                Ok(actions) => actions,
                Err(SpotifyApiError::NoToken) => vec![],
                Err(SpotifyApiError::InvalidToken) => {
                    let mut retried = call().await.unwrap_or_else(|_| Vec::new());
                    retried.insert(0, LoginAction::RefreshToken.into());
                    retried
                }
                Err(err) => {
                    error!("Spotify API error: {}", err);
                    vec![AppAction::ShowNotification(gettext(
                        // translators: This notification is the default message for unhandled errors. Logs refer to console output.
                        "An error occured. Check logs for details!",
                    ))]
                }
            }
        }))
    }
}

thread_local!(static CSS_ADDED: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new()));

pub fn display_add_css_provider(resource: &'static str) {
    CSS_ADDED.with(|set| {
        if set.borrow().contains(resource) {
            return;
        }

        set.borrow_mut().insert(resource);

        let provider = gtk::CssProvider::new();
        provider.load_from_resource(resource);

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
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
