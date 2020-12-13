use std::rc::Rc;
use std::ops::Deref;
use crate::app::backend::api::SpotifyApiClient;
use crate::app::state::{AppModel, BrowserAction, ScreenName};
use crate::app::dispatch::{Worker, ActionDispatcher};
use crate::app::models::*;
use super::SearchResults;

pub struct SearchFactory {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
    worker: Worker
}

impl SearchFactory {

    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>, worker: Worker) -> Self {
        Self { app_model, dispatcher, worker }
    }

    pub fn make_search_results(&self) -> SearchResults {
        let model = SearchResultsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        SearchResults::new(model, self.worker.clone())
    }
}

pub struct SearchResultsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl SearchResultsModel {

    fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher }
    }

    fn spotify(&self) -> Rc<dyn SpotifyApiClient> {
        Rc::clone(&self.app_model.services.spotify_api)
    }

    pub fn get_query(&self) -> Option<impl Deref<Target=String> + '_> {
        self.app_model.map_state_opt(|s| {
            Some(&s.browser_state
                .search_state()?.query)
        })
    }

    pub fn fetch_results(&self) {
        let api = self.spotify();
        if let Some(query) = self.get_query() {
            let query = query.to_owned();
            self.dispatcher.dispatch_async(Box::pin(async move {
                let albums = api.search_albums(&query[..], 0, 5).await?;
                Some(BrowserAction::SetSearchResults(albums).into())
            }))
        }
    }

    pub fn get_current_results(&self) -> Option<impl Deref<Target=Vec<AlbumDescription>> + '_> {
        self.app_model.map_state_opt(|s| {
            Some(&s.browser_state.search_state()?.album_results)
        })
    }

    pub fn open_album(&self, uri: &str) {
        self.dispatcher.dispatch(BrowserAction::NavigationPush(ScreenName::Details(uri.to_string())).into());

        let api = self.spotify();
        if let Some(id) = uri.split(":").last() {
            let id = id.to_owned();
            self.dispatcher.dispatch_async(Box::pin(async move {
                let album = api.get_album(&id).await?;
                Some(BrowserAction::SetDetails(album).into())
            }));
        }

    }
}

