use crate::app::AppEvent;
use gtk::prelude::*;

#[macro_export]
macro_rules! resource {
    ($resource:expr) => {
        concat!("/dev/alextren/Spot", $resource)
    };
}

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

mod browser;
pub use browser::*;

mod details;
pub use details::*;

mod search;
pub use search::*;

mod album;
use album::*;

mod song;
use song::*;

mod artist_details;
pub use artist_details::*;

mod now_playing;
pub use now_playing::*;

mod user_menu;
pub use user_menu::*;

pub fn screen_add_css_provider(resource: &str) {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource(resource);

    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
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
