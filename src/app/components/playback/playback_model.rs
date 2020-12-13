use std::rc::Rc;
use std::ops::Deref;

use crate::app::{AppModel, AppState, AppAction, ActionDispatcher};
use crate::app::models::*;


pub struct PlaybackModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl PlaybackModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher }
    }

    fn state(&self) -> impl Deref<Target = AppState> + '_ {
        self.app_model.get_state()
    }

    pub fn is_playing(&self) -> bool {
        let state = self.state();
        state.is_playing && state.current_song_uri.is_some()
    }

    pub fn current_song(&self) -> Option<SongDescription> {
        let state = self.state();
        if let Some(current_song_uri) = state.current_song_uri.as_ref() {
            state.playlist.iter().find(|song| song.uri == *current_song_uri).cloned()
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
        let action = if self.is_playing() {
            AppAction::Pause
        } else {
            AppAction::Play
        };
        self.dispatcher.dispatch(action);
    }


    pub fn seek_to(&self, position: u32) {
        self.dispatcher.dispatch(AppAction::Seek(position));
    }
}

