use gtk::prelude::*;
use crate::app::AppEvent;

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

mod gtypes;

pub fn screen_add_css_provider(resource: &str) {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource(resource);

    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}


pub trait EventListener {
    fn on_event(&self, _: &AppEvent) {}
}

pub trait Component {
    fn get_root_widget(&self) -> &gtk::Widget;

    fn get_children(&self) -> Option<&Vec<Box<dyn EventListener>>> {
        None
    }

    fn broadcast_event(&self, event: &AppEvent) {
        if let Some(children) = self.get_children() {
            for child in children.iter() {
                child.on_event(event);
            }
        }
    }
}

pub trait ListenerComponent: Component + EventListener {}
impl <T> ListenerComponent for T where T: Component + EventListener {}
