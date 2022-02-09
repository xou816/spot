use crate::app::ActionDispatcher;

pub struct SettingsModel {
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SettingsModel {
    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }
}
