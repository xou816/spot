use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::handle_error;
use crate::app::models::*;
use crate::app::state::BrowserAction;
use crate::app::{ActionDispatcher, AppAction, AppModel, ListStore};

pub struct UserDetailsModel {
    pub id: String,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl UserDetailsModel {
    pub fn new(id: String, app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            id,
            app_model,
            dispatcher,
        }
    }
    pub fn get_user_name(&self) -> Option<impl Deref<Target = String> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser.user_state(&self.id)?.user.as_ref())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumModel>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.user_state(&self.id)?.playlists))
    }

    pub fn load_user_details(&self, id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_user(&id[..]).await {
                Ok(user) => Some(BrowserAction::SetUserDetails(user).into()),
                Err(err) => handle_error(err),
            }
        }));
    }

    pub fn open_playlist(&self, id: &str) {
        self.dispatcher
            .dispatch(AppAction::ViewPlaylist(id.to_string()));
    }

    pub fn load_more(&self) -> Option<()> {
        let api = self.app_model.get_spotify();
        let state = self.app_model.get_state();
        let next_page = &state.browser.user_state(&self.id)?.next_page;

        let id = next_page.data.clone();
        let batch_size = next_page.batch_size;
        let offset = next_page.next_offset?;

        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_user_playlists(&id, offset, batch_size).await {
                Ok(playlists) => Some(BrowserAction::AppendUserPlaylists(playlists).into()),
                Err(err) => handle_error(err),
            }
        }));

        Some(())
    }
}
