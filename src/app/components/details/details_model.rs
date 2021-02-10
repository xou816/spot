use ref_filter_map::*;
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::backend::api::SpotifyApiError;
use crate::app::components::PlaylistModel;
use crate::app::dispatch::ActionDispatcher;
use crate::app::models::*;
use crate::app::state::{BrowserAction, BrowserEvent, DetailsState};
use crate::app::{AppAction, AppEvent, AppModel, AppState};

pub struct DetailsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl DetailsModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn get_album_info(&self) -> Option<impl Deref<Target = AlbumDescription> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser_state.details_state()?.content.as_ref())
    }

    pub fn load_album_info(&self, id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_album(&id).await {
                Ok(album) => Some(BrowserAction::SetAlbumDetails(album).into()),
                Err(SpotifyApiError::InvalidToken) => Some(AppAction::RefreshToken),
                _ => None,
            }
        }));
    }

    pub fn view_artist(&self) {
        if let Some(album) = self.get_album_info() {
            let artist = &album.artists.first().unwrap().id;
            self.dispatcher
                .dispatch(AppAction::ViewArtist(artist.to_owned()));
        }
    }
}

impl DetailsModel {
    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }

    fn details_state(&self) -> Option<Ref<'_, DetailsState>> {
        self.app_model
            .map_state_opt(|s| s.browser_state.details_state())
    }
}

impl PlaylistModel for DetailsModel {
    fn current_song_id(&self) -> Option<String> {
        self.state().current_song_id.clone()
    }

    fn songs(&self) -> Option<Ref<'_, Vec<SongDescription>>> {
        ref_filter_map(self.details_state()?, |s| Some(&s.content.as_ref()?.songs))
    }

    fn play_song(&self, id: String) {
        let full_state = self.app_model.get_state();
        let is_in_playlist = full_state.playlist.songs().iter().any(|s| s.id.eq(&id));
        if let (Some(songs), false) = (self.songs(), is_in_playlist) {
            self.dispatcher
                .dispatch(AppAction::LoadPlaylist(songs.clone()));
        }
        self.dispatcher.dispatch(AppAction::Load(id));
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::AlbumDetailsLoaded)
        )
    }

    fn actions_for(&self, _: String) -> Option<gio::ActionGroup> {
        None
    }

    fn menu_for(&self, _: String) -> Option<gio::MenuModel> {
        None
    }
}
