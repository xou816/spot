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

pub mod utils;

pub mod labels;

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
            match first_call().await {
                Ok(actions) => actions,
                Err(SpotifyApiError::NoToken) => vec![],
                Err(SpotifyApiError::InvalidToken) => {
                    let mut retried = call().await.unwrap_or_else(|_| Vec::new());
                    retried.push(LoginAction::RefreshToken.into());
                    retried
                }
                Err(err) => {
                    println!("Error: {:?}", err);
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

        gtk::StyleContext::add_provider_for_display(
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
