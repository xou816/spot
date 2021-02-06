use crate::app::backend::api::SpotifyApiError;
use crate::app::models::*;
use crate::app::{ActionDispatcher, AppAction, AppModel, BrowserAction, ListStore};
use std::ops::Deref;
use std::rc::Rc;

pub struct ArtistDetailsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl ArtistDetailsModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn get_artist_name(&self) -> Option<impl Deref<Target = String> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser_state.artist_state()?.artist.as_ref())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumModel>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser_state.artist_state()?.albums))
    }

    pub fn load_artist_details(&self, id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_artist(&id[..]).await {
                Ok(artist) => Some(BrowserAction::SetArtistDetails(artist).into()),
                Err(SpotifyApiError::InvalidToken) => Some(AppAction::RefreshToken),
                _ => None
            }
        }));
    }

    pub fn open_album(&self, id: &str) {
        self.dispatcher
            .dispatch(AppAction::ViewAlbum(id.to_string()));
    }
}
