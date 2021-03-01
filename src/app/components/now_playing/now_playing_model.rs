use gdk::SELECTION_CLIPBOARD;
use gio::prelude::*;
use gio::{ActionMapExt, SimpleAction, SimpleActionGroup};
use gtk::Clipboard;
use std::cell::Ref;
use std::rc::Rc;

use crate::app::components::PlaylistModel;
use crate::app::models::SongModel;
use crate::app::state::{PlaybackAction, PlaybackEvent, PlaybackState};
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

    fn queue(&self) -> Ref<'_, PlaybackState> {
        Ref::map(self.state(), |s| &s.playback)
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

    fn play_song(&self, id: String) {
        self.dispatcher.dispatch(PlaybackAction::Load(id).into());
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::PlaybackEvent(PlaybackEvent::PlaylistChanged)
        )
    }

    fn actions_for(&self, id: String) -> Option<gio::ActionGroup> {
        let queue = self.queue();
        let song = queue.song(&id)?;
        let group = SimpleActionGroup::new();

        let album_id = song.album.id.clone();
        let view_album = SimpleAction::new("view_album", None);
        let dispatcher = self.dispatcher.box_clone();
        view_album.connect_activate(move |_, _| {
            dispatcher.dispatch(AppAction::ViewAlbum(album_id.clone()));
        });

        group.add_action(&view_album);

        for (i, artist) in song.artists.iter().enumerate() {
            let view_artist = SimpleAction::new(&format!("view_artist_{}", i), None);
            let dispatcher = self.dispatcher.box_clone();
            let id = artist.id.clone();
            view_artist.connect_activate(move |_, _| {
                dispatcher.dispatch(AppAction::ViewArtist(id.clone()));
            });
            group.add_action(&view_artist);
        }

        let track_id = song.id.clone();
        let copy_link = SimpleAction::new("copy_link", None);
        copy_link.connect_activate(move |_, _| {
            let clipboard = Clipboard::get(&SELECTION_CLIPBOARD);
            clipboard.set_text(&format!("https://open.spotify.com/track/{}", &track_id));
        });
        group.add_action(&copy_link);

        Some(group.upcast())
    }

    fn menu_for(&self, id: String) -> Option<gio::MenuModel> {
        let queue = self.queue();
        let song = queue.song(&id)?;

        let menu = gio::Menu::new();
        menu.append(Some("View album"), Some("song.view_album"));
        for (i, artist) in song.artists.iter().enumerate() {
            menu.append(
                Some(&format!("More from {}", artist.name)),
                Some(&format!("song.view_artist_{}", i)),
            );
        }

        menu.append(Some("Copy link"), Some("song.copy_link"));

        Some(menu.upcast())
    }
}
