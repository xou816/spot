use crate::app::{ActionDispatcher, BrowserAction};

pub struct SearchBarModel(pub Box<dyn ActionDispatcher>);

impl SearchBarModel {
    pub fn search(&self, query: String) {
        self.0.dispatch(BrowserAction::Search(query).into());
    }
}
