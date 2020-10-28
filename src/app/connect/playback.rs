use std::rc::{Rc};
use std::cell::{Ref, RefCell};

use crate::app::{AppModel, AppState, AppAction, ActionDispatcher};
use crate::app::models::*;
use crate::app::components::{PlaybackModel};

pub struct PlaybackModelImpl {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl PlaybackModelImpl {
    pub fn new(app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher }
    }

    fn state(&self) -> Ref<'_, AppState> {
        Ref::map(self.app_model.borrow(), |m| &m.state)
    }
}

impl PlaybackModel for PlaybackModelImpl {

    fn is_playing(&self) -> bool {
        let state = self.state();
        state.is_playing && state.current_song_uri.is_some()
    }

    fn current_song(&self) -> Option<SongDescription> {
        let state = self.state();
        if let Some(current_song_uri) = state.current_song_uri.as_ref() {
            state.playlist.iter().find(|song| song.uri == *current_song_uri).cloned()
        } else {
            None
        }
    }

    fn play_next_song(&self) {
        self.dispatcher.dispatch(AppAction::Next);
    }

    fn play_prev_song(&self) {
        self.dispatcher.dispatch(AppAction::Previous);

    }

    fn toggle_playback(&self) {
        let action = if self.is_playing() {
            AppAction::Pause
        } else {
            AppAction::Play
        };
        self.dispatcher.dispatch(action);
    }


    fn seek_to(&self, position: u32) {
        self.dispatcher.dispatch(AppAction::Seek(position));
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use futures::executor::block_on;
    use futures::future::LocalBoxFuture;

    use crate::app::backend::api::tests::TestSpotifyApiClient;

    #[derive(Clone)]
    struct TestDispatcher(Rc<RefCell<AppModel>>);

    impl ActionDispatcher for TestDispatcher {
        fn dispatch(&self, action: AppAction) {
            self.0.borrow_mut().update_state(action);
        }

        fn dispatch_async(&self, action: LocalBoxFuture<'static, Option<AppAction>>) {
            if let Some(action) = block_on(action) {
                self.dispatch(action);
            }
        }

        fn dispatch_many_async(&self, actions: LocalBoxFuture<'static, Vec<AppAction>>) {}

        fn box_clone(&self) -> Box<dyn ActionDispatcher> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_playback() {

        let mut state = AppState::new(vec![
            SongDescription::new("Song 1", "Artist", "uri1", 1000),
            SongDescription::new("Song 2", "Artist", "uri2", 1000)
        ]);
        state.current_song_uri = Some("uri1".to_owned());

        let app_model = Rc::new(RefCell::new(AppModel::new(state, Rc::new(TestSpotifyApiClient::new()))));
        let dispatcher = Box::new(TestDispatcher(Rc::clone(&app_model)));
        let model = PlaybackModelImpl::new(Rc::clone(&app_model), dispatcher.box_clone());

        assert!(!model.is_playing());

        model.toggle_playback();
        assert!(model.is_playing());
    }

    #[test]
    fn test_next() {

        let mut state = AppState::new(vec![
            SongDescription::new("Song 1", "Artist", "uri1", 1000),
            SongDescription::new("Song 2", "Artist", "uri2", 1000)
        ]);
        state.current_song_uri = Some("uri1".to_owned());

        let app_model = Rc::new(RefCell::new(AppModel::new(state, Rc::new(TestSpotifyApiClient::new()))));
        let dispatcher = Box::new(TestDispatcher(Rc::clone(&app_model)));
        let model = PlaybackModelImpl::new(Rc::clone(&app_model), dispatcher.box_clone());

        assert_eq!(model.current_song().unwrap().title, "Song 1");

        model.play_next_song();

        assert_eq!(model.current_song().unwrap().title, "Song 2");
    }

    #[test]
    fn test_next_no_next() {

        let mut state = AppState::new(vec![
            SongDescription::new("Song 1", "Artist", "uri1", 1000),
            SongDescription::new("Song 2", "Artist", "uri2", 1000)
        ]);
        state.current_song_uri = Some("uri2".to_owned());

        let app_model = Rc::new(RefCell::new(AppModel::new(state, Rc::new(TestSpotifyApiClient::new()))));
        let dispatcher = Box::new(TestDispatcher(Rc::clone(&app_model)));
        let model = PlaybackModelImpl::new(Rc::clone(&app_model), dispatcher.box_clone());

        assert_eq!(model.current_song().unwrap().title, "Song 2");

        model.play_next_song();

        assert_eq!(model.current_song().unwrap().title, "Song 2");
    }
}


