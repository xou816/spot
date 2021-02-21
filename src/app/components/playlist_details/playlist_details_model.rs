use ref_filter_map::*;
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{handle_error, PlaylistModel};
use crate::app::dispatch::ActionDispatcher;
use crate::app::models::*;
use crate::app::state::{BrowserAction, BrowserEvent, PlaylistDetailsState};
use crate::app::{AppAction, AppEvent, AppModel, AppState};

pub struct PlaylistDetailsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl PlaylistDetailsModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn get_playlist_info(&self) -> Option<impl Deref<Target = PlaylistDescription> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser_state.playlist_details_state()?.content.as_ref())
    }

    pub fn load_playlist_info(&self, id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_playlist(&id).await {
                Ok(playlist) => Some(BrowserAction::SetPlaylistDetails(playlist).into()),
                Err(err) => handle_error(err),
            }
        }));
    }
}

impl PlaylistDetailsModel {
    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }

    fn details_state(&self) -> Option<Ref<'_, PlaylistDetailsState>> {
        self.app_model
            .map_state_opt(|s| s.browser_state.playlist_details_state())
    }
}

impl PlaylistModel for PlaylistDetailsModel {
    fn current_song_id(&self) -> Option<String> {
        self.state().current_song_id.clone()
    }

    fn songs(&self) -> Vec<SongModel> {
        let songs = self.app_model.map_state_opt(|s| {
            Some(
                &s.browser_state
                    .playlist_details_state()?
                    .content
                    .as_ref()?
                    .songs,
            )
        });
        match songs {
            Some(songs) => songs
                .iter()
                .enumerate()
                .map(|(i, s)| s.to_song_model(i))
                .collect(),
            None => vec![],
        }
    }

    fn play_song(&self, id: String) {
        let full_state = self.app_model.get_state();
        let is_in_playlist = full_state.playlist.song(&id).is_some();
        // if !is_in_playlist {
        //     self.dispatcher
        //         .dispatch(AppAction::LoadPlaylist(self.songs().cloned().collect()));
        // }
        self.dispatcher.dispatch(AppAction::Load(id));
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::PlaylistDetailsLoaded)
        )
    }

    fn actions_for(&self, _: String) -> Option<gio::ActionGroup> {
        None
    }

    fn menu_for(&self, _: String) -> Option<gio::MenuModel> {
        None
    }
}
