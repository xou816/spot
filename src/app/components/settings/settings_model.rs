use crate::app::state::SettingsAction;
use crate::app::ActionDispatcher;
use crate::player::SpotifyPlayerSettings;

pub struct SettingsModel {
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SettingsModel {
    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }

    pub fn set_player_settings(&self, settings: SpotifyPlayerSettings) {
        self.dispatcher
            .dispatch(SettingsAction::ChangePlayerSettings(settings).into());
    }
}
