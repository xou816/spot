use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{handle_error, PlaylistModel};
use crate::app::dispatch::ActionDispatcher;
use crate::app::models::*;
use crate::app::state::{BrowserAction, BrowserEvent, PlaybackAction};
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
            .map_state_opt(|s| s.browser.details_state()?.content.as_ref())
    }

    pub fn load_album_info(&self, id: &str) {
        let id = id.to_owned();
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_album(&id).await {
                Ok(album) => Some(BrowserAction::SetAlbumDetails(album).into()),
                Err(err) => handle_error(err),
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

    pub fn toggle_save_album(&self) {
        if let Some(album) = self.get_album_info() {
            let id = album.id.clone();
            let is_liked = album.is_liked;

            let api = self.app_model.get_spotify();

            self.dispatcher.dispatch_async(Box::pin(async move {
                if !is_liked {
                    match api.save_album(&id).await {
                        Ok(album) => Some(BrowserAction::SaveAlbum(album).into()),
                        Err(err) => handle_error(err),
                    }
                } else {
                    match api.remove_saved_album(&id).await {
                        Ok(_) => Some(BrowserAction::UnsaveAlbum(id).into()),
                        Err(err) => handle_error(err),
                    }
                }
            }));
        }
    }
}

impl DetailsModel {
    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }
}

impl PlaylistModel for DetailsModel {
    fn current_song_id(&self) -> Option<String> {
        self.state().playback.current_song_id.clone()
    }

    fn songs(&self) -> Vec<SongModel> {
        let songs = self
            .app_model
            .map_state_opt(|s| Some(&s.browser.details_state()?.content.as_ref()?.songs));
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
        // let full_state = self.app_model.get_state();
        // let is_in_playlist = full_state.playlist.song(&id).is_some();
        // if !is_in_playlist {
        //     self.dispatcher
        //         .dispatch(AppAction::LoadPlaylist(self.songs().cloned().collect()));
        // }
        self.dispatcher.dispatch(PlaybackAction::Load(id).into());
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::AlbumDetailsLoaded(_))
        )
    }

    fn actions_for(&self, _: String) -> Option<gio::ActionGroup> {
        None
    }

    fn menu_for(&self, _: String) -> Option<gio::MenuModel> {
        None
    }
}
