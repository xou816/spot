use std::ops::Deref;
use std::rc::Rc;

use crate::app::backend::api::SpotifyApiError;
use crate::app::dispatch::ActionDispatcher;
use crate::app::models::*;
use crate::app::state::{AppAction, AppModel, BrowserAction};

pub struct SearchResultsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SearchResultsModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn get_query(&self) -> Option<impl Deref<Target = String> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.search_state()?.query))
    }

    pub fn fetch_results(&self) {
        let api = self.app_model.get_spotify();
        if let Some(query) = self.get_query() {
            let query = query.to_owned();
            self.dispatcher.dispatch_async(Box::pin(async move {
                match api.search(&query[..], 0, 5).await {
                    Ok(albums) => Some(BrowserAction::SetSearchResults(albums).into()),
                    Err(SpotifyApiError::InvalidToken) => Some(AppAction::RefreshToken),
                    _ => None,
                }
            }))
        }
    }

    pub fn get_album_results(&self) -> Option<impl Deref<Target = Vec<AlbumDescription>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.search_state()?.album_results))
    }

    pub fn get_artist_results(&self) -> Option<impl Deref<Target = Vec<ArtistSummary>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.search_state()?.artist_results))
    }

    pub fn open_album(&self, id: &str) {
        self.dispatcher
            .dispatch(AppAction::ViewAlbum(id.to_string()));
    }

    pub fn open_artist(&self, id: &str) {
        self.dispatcher
            .dispatch(AppAction::ViewArtist(id.to_string()));
    }
}
