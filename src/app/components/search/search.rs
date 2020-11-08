use gtk::prelude::*;
use crate::app::components::{Component, EventListener};

pub struct SearchFactory {

}

impl SearchFactory {

    pub fn new() -> Self {
        Self {}
    }

    pub fn make_search_results(&self) -> SearchResults {
        SearchResults::new()
    }
}

pub struct SearchResults {
    root: gtk::Widget
}

impl SearchResults {

    pub fn new() -> Self {
        let builder = gtk::Builder::new_from_resource("/dev/alextren/Spot/components/search.ui");
        let root: gtk::Box = builder.get_object("search_root").unwrap();
        Self { root: root.upcast() }
    }
}

impl Component for SearchResults {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.root
    }
}

impl EventListener for SearchResults {}
