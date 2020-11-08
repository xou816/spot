use crate::app::{ActionDispatcher, BrowserAction};

use super::SearchBarModel;

pub struct SearchBarModelImpl(pub Box<dyn ActionDispatcher>);

impl SearchBarModel for SearchBarModelImpl {

    fn search(&self, query: String) {
        self.0.dispatch(BrowserAction::Search(query).into());
    }
}
