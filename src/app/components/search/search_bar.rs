use std::rc::Rc;
use gtk::prelude::*;
use gtk::SearchEntryExt;
use crate::app::components::EventListener;

use super::SearchBarModel;

pub struct SearchBar;

impl SearchBar {

    pub fn new(
        model: SearchBarModel,
        search_entry: gtk::SearchEntry) -> Self {

        let model = Rc::new(model);

        {
            let model = model.clone();
            search_entry.connect_search_changed(move |s| {
                let query = s.get_text().as_str().to_string();
                if !query.is_empty() {
                    model.search(query);
                }
            });
        }

        {
            let model = model.clone();
            search_entry.connect_focus_in_event(move |s, _| {
                let query = s.get_text().as_str().to_string();
                if !query.is_empty() {
                    model.search(query);
                }
                glib::signal::Inhibit(false)
            });
        }

        Self {}
    }
}

impl EventListener for SearchBar {}
