use std::rc::{Rc};
use std::cell::{Ref, RefCell};

use crate::app::{AppModel, AppState, AppAction, SongDescription};
use crate::app::components::{PlaybackModel};

pub struct PlaybackModelImpl(pub Rc<RefCell<AppModel>>);

impl PlaybackModelImpl {
    fn dispatch(&self, action: AppAction) {
        self.0.borrow().dispatch(action);
    }

    fn state(&self) -> Ref<AppState> {
        Ref::map(self.0.borrow(), |m| &m.state)
    }
}

impl PlaybackModel for PlaybackModelImpl {

    fn is_playing(&self) -> bool {
        self.state().is_playing
            && self.state().current_song_uri.is_some()
    }

    fn current_song(&self) -> Option<SongDescription> {
        let state = self.state();
        if let Some(current_song_uri) = state.current_song_uri.as_ref() {
            state.playlist.iter().find(|&song| song.uri == *current_song_uri).cloned()
        } else {
            Option::None
        }

    }

    fn play_next_song(&self) {
        let state = self.state();
        let next_song = state.current_song_uri.as_ref().and_then(|uri| {
            state.playlist.iter()
                .skip_while(|&song| song.uri != *uri)
                .skip(1)
                .next()
        });
        if let Some(song) = next_song {
            let action = AppAction::Load(song.uri.clone());
            self.dispatch(action);
        }
    }

    fn play_prev_song(&self) {
        let state = self.state();
        let prev_song = state.current_song_uri.as_ref().and_then(|uri| {
            state.playlist.iter()
                .take_while(|&song| song.uri != *uri)
                .last()
        });
        if let Some(song) = prev_song {
            let action = AppAction::Load(song.uri.clone());
            self.dispatch(action);
        }

    }

    fn toggle_playback(&self) {
        let action = if self.is_playing() {
            AppAction::Pause
        } else {
            AppAction::Play
        };
        self.dispatch(action);
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::rc::Weak;
    use crate::app::dispatch::{AbstractDispatcher};

    struct TestDispatcher(pub Weak<RefCell<AppModel>>);

    impl AbstractDispatcher<AppAction> for TestDispatcher {
        fn dispatch(&self, action: AppAction) -> Option<()> {
            self.0.upgrade().unwrap().borrow_mut().update_state(&action);
            Some(())
        }
    }

    fn make_app_model(state: AppState) -> Rc<RefCell<AppModel>> {
        let app_model = AppModel::new(state, Rc::new(TestDispatcher(Weak::default())));
        let app_model = Rc::new(RefCell::new(app_model));
        app_model.borrow_mut().dispatcher = Rc::new(TestDispatcher(Rc::downgrade(&app_model)));
        app_model
    }

    #[test]
    fn test_playback() {

        let state = AppState::new(vec![
            SongDescription::new("Song 1", "Artist", "uri1"),
            SongDescription::new("Song 2", "Artist", "uri2")
        ]);

        let app_model = make_app_model(state);
        let model = PlaybackModelImpl(app_model);


        assert!(!model.is_playing());
        //model.toggle_playback();
        assert!(model.is_playing());
    }
}


