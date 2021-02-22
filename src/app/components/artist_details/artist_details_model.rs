use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{handle_error, PlaylistModel};
use crate::app::models::*;
use crate::app::state::{BrowserAction, BrowserEvent, PlaybackAction};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, ListStore};

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
            .map_state_opt(|s| s.browser.artist_state()?.artist.as_ref())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumModel>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.artist_state()?.albums))
    }

    pub fn load_artist_details(&self, id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_artist(&id[..]).await {
                Ok(artist) => Some(BrowserAction::SetArtistDetails(artist).into()),
                Err(err) => handle_error(err),
            }
        }));
    }

    pub fn open_album(&self, id: &str) {
        self.dispatcher
            .dispatch(AppAction::ViewAlbum(id.to_string()));
    }

    pub fn load_more(&self) -> Option<()> {
        let api = self.app_model.get_spotify();
        let state = self.app_model.get_state();
        let next_page = &state.browser.artist_state()?.next_page;

        let id = next_page.data.clone();
        let batch_size = next_page.batch_size;
        let offset = next_page.next_offset?;

        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_artist_albums(&id, offset, batch_size).await {
                Ok(albums) => Some(BrowserAction::AppendArtistReleases(albums).into()),
                Err(err) => handle_error(err),
            }
        }));

        Some(())
    }
}

impl PlaylistModel for ArtistDetailsModel {
    fn songs(&self) -> Vec<SongModel> {
        let tracks = self
            .app_model
            .map_state_opt(|s| Some(&s.browser.artist_state()?.top_tracks));

        match tracks {
            Some(tracks) => tracks
                .iter()
                .enumerate()
                .map(|(i, s)| s.to_song_model(i))
                .collect(),
            None => vec![],
        }
    }

    fn current_song_id(&self) -> Option<String> {
        self.app_model.get_state().playback.current_song_id.clone()
    }

    fn play_song(&self, id: String) {
        let full_state = self.app_model.get_state();
        let is_in_playlist = full_state.playback.song(&id).is_some();
        // if !is_in_playlist {
        //     self.dispatcher
        //         .dispatch(AppAction::LoadPlaylist(self.songs().cloned().collect()));
        // }
        self.dispatcher.dispatch(PlaybackAction::Load(id).into());
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::ArtistDetailsUpdated)
        )
    }

    fn actions_for(&self, _: String) -> Option<gio::ActionGroup> {
        None
    }

    fn menu_for(&self, _: String) -> Option<gio::MenuModel> {
        None
    }
}
