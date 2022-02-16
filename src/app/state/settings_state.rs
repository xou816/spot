use crate::app::state::{AppAction, AppEvent, UpdatableState};
use crate::player::SpotifyPlayerSettings;

#[derive(Clone, Debug)]
pub enum SettingsAction {
    ChangePlayerSettings(SpotifyPlayerSettings),
}

impl From<SettingsAction> for AppAction {
    fn from(settings_action: SettingsAction) -> Self {
        Self::SettingsAction(settings_action)
    }
}

#[derive(Clone, Debug)]
pub enum SettingsEvent {
    PlayerSettingsChanged(SpotifyPlayerSettings),
}

impl From<SettingsEvent> for AppEvent {
    fn from(settings_event: SettingsEvent) -> Self {
        Self::SettingsEvent(settings_event)
    }
}

#[derive(Default)]
pub struct SettingsState {}

impl UpdatableState for SettingsState {
    type Action = SettingsAction;
    type Event = AppEvent;

    fn update_with(&mut self, action: std::borrow::Cow<Self::Action>) -> Vec<Self::Event> {
        match action.into_owned() {
            SettingsAction::ChangePlayerSettings(settings) => {
                vec![SettingsEvent::PlayerSettingsChanged(settings).into()]
            }
        }
    }
}
