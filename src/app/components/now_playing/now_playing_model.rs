use gio::prelude::*;
use gio::{ActionMapExt, SimpleAction, SimpleActionGroup};
use std::cell::Ref;
use std::rc::Rc;

use crate::app::components::PlaylistModel;
use crate::app::models::SongDescription;
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, AppState};

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
}

impl PlaylistModel for NowPlayingModel {
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

    fn actions_for(&self, id: String) -> Option<gio::ActionGroup> {
        let songs = self.songs()?;
        let song = songs.iter().find(|s| s.id.eq(&id))?;

        let album_id = song.album.id.clone();
        let view_album = SimpleAction::new("view_album", None);
        let dispatcher = self.dispatcher.box_clone();
        view_album.connect_activate(move |_, _| {
            dispatcher.dispatch(AppAction::ViewAlbum(album_id.clone()));
        });

        let group = SimpleActionGroup::new();
        group.add_action(&view_album);
        Some(group.upcast())
    }

    fn menu_for(&self, _: String) -> Option<gio::MenuModel> {
        let menu = gio::Menu::new();
        menu.insert(0, Some("View album"), Some("song.view_album"));
        Some(menu.upcast())
    }
}
