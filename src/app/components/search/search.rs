use gio::prelude::*;
use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::utils::{wrap_flowbox_item, Debouncer};
use crate::app::components::{Album, Artist, Component, EventListener};
use crate::app::dispatch::Worker;
use crate::app::models::{AlbumModel, ArtistModel};
use crate::app::state::{AppEvent, BrowserEvent};

use super::SearchResultsModel;

#[derive(Gladis, Clone)]
struct SearchResultsWidget {
    search_root: gtk::Widget,
    results_label: gtk::Label,
    albums_results: gtk::FlowBox,
    artist_results: gtk::FlowBox,
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
    artist_results_model: gio::ListStore,
    debouncer: Debouncer,
}

impl SearchResults {
    pub fn new(model: SearchResultsModel, worker: Worker) -> Self {
        let model = Rc::new(model);
        let widget = SearchResultsWidget::new();

        let album_results_model = gio::ListStore::new(AlbumModel::static_type());
        let artist_results_model = gio::ListStore::new(ArtistModel::static_type());

        let model_clone = Rc::downgrade(&model);
        let worker_clone = worker.clone();
        widget
            .albums_results
            .bind_model(Some(&album_results_model), move |item| {
                wrap_flowbox_item(item, |item: &AlbumModel| {
                    let album = Album::new(item, worker_clone.clone());
                    let weak = model_clone.clone();
                    album.connect_album_pressed(move |a| {
                        if let (Some(id), Some(m)) = (a.uri().as_ref(), weak.upgrade()) {
                            m.open_album(id);
                        }
                    });
                    album.get_root_widget().clone()
                })
            });

        let model_clone = Rc::downgrade(&model);
        widget
            .artist_results
            .bind_model(Some(&artist_results_model), move |item| {
                wrap_flowbox_item(item, |item: &ArtistModel| {
                    let artist = Artist::new(item, worker.clone());
                    let weak = model_clone.clone();
                    artist.connect_artist_pressed(move |a| {
                        if let (Some(id), Some(m)) = (a.id().as_ref(), weak.upgrade()) {
                            m.open_artist(id);
                        }
                    });
                    artist.get_root_widget().clone()
                })
            });

        Self {
            widget,
            model,
            album_results_model,
            artist_results_model,
            debouncer: Debouncer::new(),
        }
    }

    fn update_results(&self) {
        if let Some(results) = self.model.get_album_results() {
            self.album_results_model.remove_all();
            for album in results.iter() {
                self.album_results_model.append(&AlbumModel::new(
                    &album.artists_name(),
                    &album.title,
                    &album.art,
                    &album.id,
                ));
            }
        }
        if let Some(results) = self.model.get_artist_results() {
            self.artist_results_model.remove_all();
            for artist in results.iter() {
                self.artist_results_model.append(&ArtistModel::new(
                    &artist.name,
                    &artist.photo,
                    &artist.id,
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
    fn on_event(&mut self, app_event: &AppEvent) {
        match app_event {
            AppEvent::BrowserEvent(BrowserEvent::SearchUpdated) => {
                self.update_search_query();
            }
            AppEvent::BrowserEvent(BrowserEvent::SearchResultsUpdated) => {
                self.update_results();
            }
            _ => {}
        }
    }
}
