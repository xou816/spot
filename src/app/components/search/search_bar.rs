use gtk::prelude::*;
use libhandy::traits::SearchBarExt;
use std::rc::Rc;

use super::SearchBarModel;
use crate::app::components::EventListener;

pub struct SearchBar;

impl SearchBar {
    pub fn new(
        model: SearchBarModel,
        search_button: gtk::ToggleButton,
        search_bar: libhandy::SearchBar,
        search_entry: gtk::SearchEntry,
    ) -> Self {
        let model = Rc::new(model);

        {
            let model = model.clone();
            search_entry.connect_changed(move |s| {
                let query = s.text().as_str().to_string();
                if !query.is_empty() {
                    model.search(query);
                }
            });
        }

        search_entry.connect_focus_in_event(move |s, _| {
            let query = s.text().as_str().to_string();
            if !query.is_empty() {
                model.search(query);
            }
            Inhibit(false)
        });

        search_button.connect_clicked(clone!(@weak search_bar => move |b| {
            search_bar.set_search_mode(b.is_active());
        }));

        search_bar.connect_search_mode_enabled_notify(clone!(@weak search_button => move |s| {
            let active = s.is_search_mode();
            if active != search_button.is_active() {
                search_button.set_active(active);
            }
        }));

        search_bar.connect_entry(&search_entry);

        Self
    }
}

impl EventListener for SearchBar {}
