use gio::prelude::*;
use gio::{ActionMapExt, SimpleActionGroup};
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::PlaylistModel;
use crate::app::models::SongModel;
use crate::app::state::{
    PlaybackAction, PlaybackEvent, PlaybackState, SelectionAction, SelectionState,
};
use crate::app::{ActionDispatcher, AppEvent, AppModel, AppState};

pub struct NowPlayingModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl NowPlayingModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }

    fn queue(&self) -> Ref<'_, PlaybackState> {
        Ref::map(self.state(), |s| &s.playback)
    }

    pub fn toggle_shuffle(&self) {
        self.dispatcher
            .dispatch(PlaybackAction::ToggleShuffle.into());
    }

    pub fn clear_queue(&self) {
        self.dispatcher.dispatch(PlaybackAction::ClearQueue.into());
    }
}

impl PlaylistModel for NowPlayingModel {
    fn current_song_id(&self) -> Option<String> {
        self.queue().current_song_id.clone()
    }

    fn songs(&self) -> Vec<SongModel> {
        self.queue()
            .songs()
            .enumerate()
            .map(|(i, s)| s.to_song_model(i))
            .collect()
    }

    fn play_song(&self, id: &str) {
        self.dispatcher
            .dispatch(PlaybackAction::Load(id.to_string()).into());
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::PlaybackEvent(PlaybackEvent::PlaylistChanged)
        )
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let queue = self.queue();
        let song = queue.song(id)?;
        let group = SimpleActionGroup::new();

        for view_artist in song.make_artist_actions(self.dispatcher.box_clone(), None) {
            group.add_action(&view_artist);
        }
        group.add_action(&song.make_album_action(self.dispatcher.box_clone(), None));
        group.add_action(&song.make_link_action(None));
        group.add_action(&song.make_dequeue_action(self.dispatcher.box_clone(), None));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let queue = self.queue();
        let song = queue.song(id)?;

        let menu = gio::Menu::new();
        menu.append(Some("View album"), Some("song.view_album"));
        for artist in song.artists.iter() {
            menu.append(
                Some(&format!("More from {}", artist.name)),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some("Copy link"), Some("song.copy_link"));
        menu.append(Some("Dequeue"), Some("song.dequeue"));

        Some(menu.upcast())
    }

    fn select_song(&self, id: &str) {
        let queue = self.queue();
        if let Some(song) = queue.song(id) {
            self.dispatcher
                .dispatch(SelectionAction::Select(song.clone()).into());
        }
    }

    fn deselect_song(&self, id: &str) {
        self.dispatcher
            .dispatch(SelectionAction::Deselect(id.to_string()).into());
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        Some(Box::new(self.app_model.map_state(|s| &s.selection)))
    }
}
