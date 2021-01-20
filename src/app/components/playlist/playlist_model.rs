use gio::prelude::*;
use ref_filter_map::*;
use std::cell::Ref;
use std::rc::Rc;

use crate::app::models::*;
use crate::app::state::DetailsState;
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, AppState, BrowserEvent};

use super::{Playlist, PlaylistModel};

pub struct PlaylistFactory {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl Clone for PlaylistFactory {
    fn clone(&self) -> Self {
        Self::new(Rc::clone(&self.app_model), self.dispatcher.box_clone())
    }
}

impl PlaylistFactory {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn make_current_playlist(&self, listbox: gtk::ListBox) -> Playlist {
        let model =
            CurrentlyPlayingModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Playlist::new(listbox, Rc::new(model))
    }

    pub fn make_custom_playlist(&self, listbox: gtk::ListBox) -> Playlist {
        let model = AlbumDetailsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Playlist::new(listbox, Rc::new(model))
    }
}

struct CurrentlyPlayingModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl CurrentlyPlayingModel {
    fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }
}

impl PlaylistModel for CurrentlyPlayingModel {
    fn current_song_id(&self) -> Option<String> {
        self.state().current_song_id.clone()
    }

    fn songs(&self) -> Option<Ref<'_, Vec<SongDescription>>> {
        Some(Ref::map(self.state(), |s| s.playlist.songs()))
    }

    fn play_song(&self, id: String) {
        self.dispatcher.dispatch(AppAction::Load(id));
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(event, AppEvent::PlaylistChanged)
    }

    fn menu_for(&self, _: String) -> Option<gio::MenuModel> {
        let menu = gio::Menu::new();
        menu.insert(0, Some("View album"), None);
        Some(menu.upcast())
    }
}

struct AlbumDetailsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl AlbumDetailsModel {
    fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }

    fn details_state(&self) -> Option<Ref<'_, DetailsState>> {
        self.app_model
            .map_state_opt(|s| s.browser_state.details_state())
    }
}

impl PlaylistModel for AlbumDetailsModel {
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
        matches!(event, AppEvent::BrowserEvent(BrowserEvent::DetailsLoaded))
    }

    fn menu_for(&self, _: String) -> Option<gio::MenuModel> {
        None
    }
}
