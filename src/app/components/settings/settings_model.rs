use crate::{app::ActionDispatcher, settings::SpotSettings};

pub struct SettingsModel {
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SettingsModel {
    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }

    pub fn save(&self, settings: SpotSettings) {}
}
