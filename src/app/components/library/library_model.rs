use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::backend::api::SpotifyApiError;
use crate::app::models::*;
use crate::app::state::HomeState;
use crate::app::{ActionDispatcher, AppAction, AppModel, BrowserAction, ListStore};

pub struct LibraryModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
    batch_size: u32,
}

impl LibraryModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
            batch_size: 20,
        }
    }

    fn state(&self) -> Option<Ref<'_, HomeState>> {
        self.app_model
            .map_state_opt(|s| s.browser_state.home_state())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumModel>> + '_> {
        Some(Ref::map(self.state()?, |s| &s.albums))
    }

    pub fn refresh_saved_albums(&self) {
        let api = self.app_model.get_spotify();
        let batch_size = self.batch_size;

        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_saved_albums(0, batch_size).await {
                Ok(albums) => Some(BrowserAction::SetLibraryContent(albums).into()),
                Err(SpotifyApiError::InvalidToken) => Some(AppAction::RefreshToken),
                _ => None,
            }
        }));
    }

    pub fn load_more_albums(&self) {
        let api = self.app_model.get_spotify();
        let page = self.state().map(|s| s.albums_page).unwrap_or(0);
        let offset = page * self.batch_size;
        let batch_size = self.batch_size;
        let current_len = self.get_list_store().unwrap().len() as u32;
        if current_len < offset {
            return;
        }

        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_saved_albums(offset, batch_size).await {
                Ok(albums) => Some(BrowserAction::AppendLibraryContent(albums).into()),
                Err(SpotifyApiError::InvalidToken) => Some(AppAction::RefreshToken),
                _ => None,
            }
        }));
    }

    pub fn open_album(&self, album_id: String) {
        self.dispatcher.dispatch(AppAction::ViewAlbum(album_id));
    }
}
