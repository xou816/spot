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
        self.dispatch(AppAction::Next);
    }

    fn play_prev_song(&self) {
        self.dispatch(AppAction::Previous);

    }

    fn toggle_playback(&self) {
        let action = if self.is_playing() {
            AppAction::Pause
        } else {
            AppAction::Play
        };
        self.dispatch(action);
    }


    fn seek_to(&self, position: u32) {
        self.dispatch(AppAction::Seek(position));
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::app::dispatch::{AbstractDispatcher};

    struct TestDispatcher(RefCell<Vec<AppAction>>);

    impl TestDispatcher {
        pub fn new() -> Self {
            Self(RefCell::new(vec![]))
        }

        pub fn flush(&self, model: Rc<RefCell<AppModel>>) {
            let mut buffer = self.0.borrow_mut();
            for action in buffer.drain(..) {
                model.borrow_mut().update_state(action);
            }
        }
    }

    impl AbstractDispatcher<AppAction> for TestDispatcher {
        fn dispatch(&self, action: AppAction) -> Option<()> {
            self.0.borrow_mut().push(action);
            Some(())
        }
    }

    fn make_model_and_dispatcher(state: AppState) -> (Rc<RefCell<AppModel>>, Rc<TestDispatcher>) {
        let dispatcher = Rc::new(TestDispatcher::new());
        let app_model = AppModel::new(state, Rc::clone(&dispatcher) as Rc<dyn AbstractDispatcher<AppAction>>);
        let app_model = Rc::new(RefCell::new(app_model));
        (app_model, dispatcher)
    }

    #[test]
    fn test_playback() {

        let mut state = AppState::new(vec![
            SongDescription::new("Song 1", "Artist", "uri1", 1000),
            SongDescription::new("Song 2", "Artist", "uri2", 1000)
        ]);
        state.current_song_uri = Some("uri1".to_owned());

        let (app_model, dispatcher) = make_model_and_dispatcher(state);
        let model = PlaybackModelImpl(Rc::clone(&app_model));

        assert!(!model.is_playing());

        model.toggle_playback();
        dispatcher.flush(app_model);

        assert!(model.is_playing());
    }

    #[test]
    fn test_next() {

        let mut state = AppState::new(vec![
            SongDescription::new("Song 1", "Artist", "uri1", 1000),
            SongDescription::new("Song 2", "Artist", "uri2", 1000)
        ]);
        state.current_song_uri = Some("uri1".to_owned());

        let (app_model, dispatcher) = make_model_and_dispatcher(state);
        let model = PlaybackModelImpl(Rc::clone(&app_model));

        assert_eq!(model.current_song().unwrap().title, "Song 1");

        model.play_next_song();
        dispatcher.flush(app_model);

        assert_eq!(model.current_song().unwrap().title, "Song 2");
    }

    #[test]
    fn test_next_no_next() {

        let mut state = AppState::new(vec![
            SongDescription::new("Song 1", "Artist", "uri1", 1000),
            SongDescription::new("Song 2", "Artist", "uri2", 1000)
        ]);
        state.current_song_uri = Some("uri2".to_owned());

        let (app_model, dispatcher) = make_model_and_dispatcher(state);
        let model = PlaybackModelImpl(Rc::clone(&app_model));

        assert_eq!(model.current_song().unwrap().title, "Song 2");

        model.play_next_song();
        dispatcher.flush(app_model);

        assert_eq!(model.current_song().unwrap().title, "Song 2");
    }
}


