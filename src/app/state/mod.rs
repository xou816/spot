mod app_model;
mod app_state;
mod browser_state;
mod login_state;
mod pagination;
mod playback_state;
mod screen_states;
mod selection_state;
mod settings_state;

use std::borrow::Cow;

pub use app_model::AppModel;
pub use app_state::*;
pub use browser_state::*;
pub use login_state::*;
pub use pagination::*;
pub use playback_state::*;
pub use screen_states::*;
pub use selection_state::*;
pub use settings_state::*;

pub trait UpdatableState {
    type Action: Clone;
    type Event;

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event>;
}
