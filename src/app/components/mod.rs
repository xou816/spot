use crate::app::AppEvent;

pub mod navigation;
pub use navigation::*;

pub mod playback;
pub use playback::*;

pub mod playlist;
pub use playlist::*;

pub mod login;
pub use login::*;

pub mod player_notifier;
pub use player_notifier::PlayerNotifier;

pub mod browser;
pub use browser::*;

pub mod details;
pub use details::*;

mod gtypes;

pub trait EventListener {
    fn on_event(&self, _: &AppEvent) {}
}

pub trait Component {
    fn get_root_widget(&self) -> &gtk::Widget;

    fn get_children(&self) -> Option<Vec<Box<dyn EventListener>>> {
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
