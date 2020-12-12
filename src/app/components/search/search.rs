use std::rc::Rc;
use std::cell::Cell;
use gtk::prelude::*;
use gio::prelude::*;
use gladis::Gladis;
use crate::app::state::{AppEvent, BrowserEvent};
use crate::app::dispatch::Worker;
use crate::app::components::{Component, EventListener, Album};
use crate::app::models::AlbumModel;
use super::SearchResultsModel;

struct Debouncer(Rc<Cell<Option<glib::source::SourceId>>>);

impl Debouncer {

    fn new() -> Self {
        Self(Rc::new(Cell::new(None)))
    }

    fn debounce<F: Fn() + 'static>(&self, interval_ms: u32, f: F) {
        let source_clone = Rc::downgrade(&self.0);
        let new_source = glib::timeout_add_local(interval_ms, move || {
            f();
            if let Some(cell) = source_clone.upgrade() {
                cell.set(None);
            }
            glib::Continue(false)
        });
        if let Some(previous_source) = self.0.replace(Some(new_source)) {
            glib::source_remove(previous_source);
        }
    }
}

#[derive(Gladis, Clone)]
struct SearchResultsWidget {
    search_root: gtk::Widget,
    results_label: gtk::Label,
    albums_results: gtk::FlowBox
}

impl SearchResultsWidget {

    fn new() -> Self {
        Self::from_resource(resource!("/components/search.ui")).unwrap()
    }
}

pub struct SearchResults {
    widget: SearchResultsWidget,
    model: Rc<SearchResultsModel>,
    album_results_model: gio::ListStore,
    debouncer: Debouncer
}

impl SearchResults {

    pub fn new(model: SearchResultsModel, worker: Worker) -> Self {
        let model = Rc::new(model);
        let widget = SearchResultsWidget::new();

        let album_results_model = gio::ListStore::new(AlbumModel::static_type());
        let worker_clone = worker.clone();

        let model_clone = Rc::clone(&model);
        widget.albums_results.bind_model(Some(&album_results_model), move |item| {
            let item = item.downcast_ref::<AlbumModel>().unwrap();
            let child = gtk::FlowBoxChild::new();
            let album = Album::new(item, worker_clone.clone());
            let weak = Rc::downgrade(&model_clone);
            album.connect_album_pressed(move |a| {
                if let (Some(uri), Some(m)) = (a.uri().as_ref(), weak.upgrade()) {
                    m.open_album(uri);
                }
            });
            child.add(album.get_root_widget());
            child.show_all();
            child.upcast::<gtk::Widget>()
        });

        Self { widget, model, album_results_model, debouncer: Debouncer::new() }
    }

    fn update_results(&self) {
        if let Some(results) = self.model.get_current_results() {
            self.album_results_model.remove_all();
            for album in results.iter() {
                self.album_results_model.append(&AlbumModel::new(
                    &album.artist,
                    &album.title,
                    &album.art,
                    &album.uri
                ));
            }
        }
    }

    fn update_search_query(&self) {

        {
            let model = Rc::downgrade(&self.model);
            self.debouncer.debounce(600, move || {
                if let Some(model) = model.upgrade() {
                    model.fetch_results();
                }
            });
        }

        if let Some(query) = self.model.get_query() {
            let formatted = format!("Search results for « {} »", *query);
            self.widget.results_label.set_label(&formatted[..]);
        }
    }
}

impl Component for SearchResults {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.search_root
    }
}

impl EventListener for SearchResults {

    fn on_event(&self, app_event: &AppEvent) {
        match app_event {
            AppEvent::BrowserEvent(BrowserEvent::SearchUpdated) => {
                self.update_search_query();
            },
            AppEvent::BrowserEvent(BrowserEvent::SearchResultsUpdated) => {
                self.update_results();
            },
            _ => {}
        }
    }
}
