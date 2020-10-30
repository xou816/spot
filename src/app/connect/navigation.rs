use crate::app::{AppModel, BrowserAction, ActionDispatcher};
use crate::app::components::NavigationModel;

pub struct NavigationModelImpl {
    dispatcher: Box<dyn ActionDispatcher>
}

impl NavigationModelImpl {

    pub fn new(dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { dispatcher }
    }
}

impl NavigationModel for NavigationModelImpl {

    fn go_back(&self) {
        self.dispatcher.dispatch(BrowserAction::GoBack.into())
    }
}
