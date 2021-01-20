use std::ops::Deref;
use std::rc::Rc;

use crate::app::models::*;
use crate::app::{ActionDispatcher, AppAction, AppModel, AppState};

pub struct PlaybackModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl PlaybackModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> impl Deref<Target = AppState> + '_ {
        self.app_model.get_state()
    }

    pub fn is_playing(&self) -> bool {
        let state = self.state();
        state.is_playing && state.current_song_id.is_some()
    }

    pub fn current_song(&self) -> Option<SongDescription> {
        let state = self.state();
        if let Some(current_song_id) = state.current_song_id.as_ref() {
            state
                .playlist
                .songs()
                .iter()
                .find(|song| song.id == *current_song_id)
                .cloned()
        } else {
            None
        }
    }

    pub fn play_next_song(&self) {
        self.dispatcher.dispatch(AppAction::Next);
    }

    pub fn play_prev_song(&self) {
        self.dispatcher.dispatch(AppAction::Previous);
    }

    pub fn toggle_playback(&self) {
        self.dispatcher.dispatch(AppAction::TogglePlay);
    }

    pub fn toggle_shuffle(&self) {
        self.dispatcher.dispatch(AppAction::ToggleShuffle);
    }

    pub fn seek_to(&self, position: u32) {
        self.dispatcher.dispatch(AppAction::Seek(position));
    }
}
