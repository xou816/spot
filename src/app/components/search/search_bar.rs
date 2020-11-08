use std::rc::Rc;
use gtk::prelude::*;
use gtk::SearchEntryExt;
use crate::app::components::EventListener;

pub trait SearchBarModel {
    fn search(&self, query: String);
}

pub struct SearchBar {
}

impl SearchBar {

    pub fn new(
        model: Rc<dyn SearchBarModel>,
        search_entry: gtk::SearchEntry) -> Self {

        {
            let model = model.clone();
            search_entry.connect_search_changed(move |s| {
                let query = s.get_text().unwrap().as_str().to_string();
                if !query.is_empty() {
                    model.search(query);
                }
            });
        }

        Self {}
    }
}

impl EventListener for SearchBar {}
