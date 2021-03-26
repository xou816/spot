use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crate::app::models::*;
use crate::app::state::HomeState;
use crate::app::{ActionDispatcher, AppAction, AppModel, BrowserAction, ListStore};

pub struct SavedPlaylistsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SavedPlaylistsModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> Option<Ref<'_, HomeState>> {
        self.app_model.map_state_opt(|s| s.browser.home_state())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumModel>> + '_> {
        Some(Ref::map(self.state()?, |s| &s.playlists))
    }

    pub fn refresh_saved_playlists(&self) -> Option<()> {
        let api = self.app_model.get_spotify();
        let batch_size = self.state()?.next_playlists_page.batch_size as u32;

        self.dispatcher.dispatch_spotify_call(move || {
            let api = Arc::clone(&api);
            async move {
                api.get_saved_playlists(0, batch_size)
                    .await
                    .map(|playlists| BrowserAction::SetPlaylistsContent(playlists).into())
            }
        });

        Some(())
    }

    pub fn load_more_playlists(&self) -> Option<()> {
        let api = self.app_model.get_spotify();

        let next_page = &self.state()?.next_playlists_page;
        let batch_size = next_page.batch_size as u32;
        let offset = next_page.next_offset? as u32;

        self.dispatcher.dispatch_spotify_call(move || {
            let api = Arc::clone(&api);
            async move {
                api.get_saved_playlists(offset, batch_size)
                    .await
                    .map(|playlists| BrowserAction::AppendPlaylistsContent(playlists).into())
            }
        });

        Some(())
    }

    pub fn open_playlist(&self, id: String) {
        self.dispatcher.dispatch(AppAction::ViewPlaylist(id));
    }
}
