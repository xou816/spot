use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::models::*;
use crate::app::state::HomeState;
use crate::app::{ActionDispatcher, AppAction, AppModel, BrowserAction, ListStore};

pub struct FollowedArtistsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl FollowedArtistsModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> Option<Ref<'_, HomeState>> {
        self.app_model.map_state_opt(|s| s.browser.home_state())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<ArtistModel>> + '_> {
        Some(Ref::map(self.state()?, |s| &s.followed_artists))
    }

    pub fn refresh_followed_artists(&self) -> Option<()> {
        let api = self.app_model.get_spotify();
        let batch_size = self.state()?.next_followed_artists_page.batch_size;

        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.get_followed_artists(0, batch_size)
                    .await
                    .map(|artists| BrowserAction::SetFollowedArtistsContent(artists.artists.into_iter().map(|artist| ArtistDescription {
                        id: artist.id,
                        name: artist.name,
                        albums: Vec::new(),
                        top_tracks: Vec::new(),
                    }).collect()).into())
            });

        Some(())
    }

    pub fn has_followed_artists(&self) -> bool {
        self.get_list_store()
            .map(|list| list.len() > 0)
            .unwrap_or(false)
    }

    pub fn load_more_followed_artists(&self) -> Option<()> {
        let api = self.app_model.get_spotify();

        let next_page = &self.state()?.next_followed_artists_page;
        let batch_size = next_page.batch_size;
        let offset = next_page.next_offset?;

        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.get_followed_artists(offset, batch_size)
                    .await
                    .map(|artists| BrowserAction::AppendFollowedArtistsContent(artists.artists.into_iter().map(|artist| ArtistDescription {
                        id: artist.id,
                        name: artist.name,
                        albums: Vec::new(),
                        top_tracks: Vec::new(),
                    }).collect()).into())
            });

        Some(())
    }

    pub fn open_artist(&self, id: String) {
        self.dispatcher.dispatch(AppAction::ViewArtist(id));
    }
}
