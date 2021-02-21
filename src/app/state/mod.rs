mod app_model;
mod app_state;
mod browser_state;
mod screen_states;
mod playback_state;

pub use app_model::AppModel;
pub use app_state::*;
pub use browser_state::*;
pub use screen_states::*;
pub use playback_state::*;

pub trait UpdatableState {
    type Action;
    type Event;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event>;
}
