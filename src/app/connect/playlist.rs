use std::rc::{Rc};
use std::cell::{Ref, RefCell};

use crate::app::{AppModel, AppState, AppAction, ActionDispatcher, SongDescription};
use crate::app::components::{Playlist, PlaylistModel};
use crate::app::browser_state::{BrowserScreen, DetailsState};

pub struct PlaylistFactory {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl PlaylistFactory {

    pub fn new(
        app_model: Rc<RefCell<AppModel>>,
        dispatcher: Box<dyn ActionDispatcher>) -> Self {

        Self { app_model, dispatcher }
    }

    pub fn make_current_playlist(&self, listbox: gtk::ListBox) -> Playlist {
        let model = CurrentlyPlayingModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Playlist::new(listbox, Rc::new(model))
    }

    pub fn make_custom_playlist(&self, listbox: gtk::ListBox) -> Playlist {
        let model = AlbumDetailsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Playlist::new(listbox, Rc::new(model))
    }
}

struct CurrentlyPlayingModel {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl CurrentlyPlayingModel {

    fn new(app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher }
    }

    fn state(&self) -> Ref<'_, AppState> {
        Ref::map(self.app_model.borrow(), |m| &m.state)
    }
}


impl PlaylistModel for CurrentlyPlayingModel {

    fn current_song_uri(&self) -> Option<String> {
        self.state().current_song_uri.clone()
    }

    fn songs(&self) -> Option<Ref<'_, Vec<SongDescription>>> {
        Some(Ref::map(self.state(), |s| &s.playlist))
    }

    fn play_song(&self, uri: String) {
        self.dispatcher.dispatch(AppAction::Load(uri));
    }
}


struct AlbumDetailsModel {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl AlbumDetailsModel {

    fn new(app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher }
    }

    fn state(&self) -> Ref<'_, AppState> {
        Ref::map(self.app_model.borrow(), |m| &m.state)
    }

    fn details_state(&self) -> Option<Ref<'_, DetailsState>> {
        let model = self.app_model.borrow();
        if model.state.browser_state.details_state().is_some() {
            Some(Ref::map(self.app_model.borrow(), |m| m.state.browser_state.details_state().unwrap()))
        } else {
            None
        }
    }
}


impl PlaylistModel for AlbumDetailsModel {

    fn current_song_uri(&self) -> Option<String> {
        self.state().current_song_uri.clone()
    }

    fn songs(&self) -> Option<Ref<'_, Vec<SongDescription>>> {
        Some(Ref::map(self.details_state()?, |s| &s.content.as_ref().unwrap().songs))
    }

    fn play_song(&self, uri: String) {
        self.dispatcher.dispatch(AppAction::Load(uri));
    }
}

