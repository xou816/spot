use gdk::SELECTION_CLIPBOARD;
use gio::prelude::*;
use gio::{ActionMapExt, SimpleAction, SimpleActionGroup};
use gtk::Clipboard;
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{handle_error, PlaylistModel};
use crate::app::models::*;
use crate::app::state::{BrowserAction, BrowserEvent, PlaybackAction, PlaylistSource};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, AppState};

pub struct PlaylistDetailsModel {
    pub id: String,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl PlaylistDetailsModel {
    pub fn new(id: String, app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            id,
            app_model,
            dispatcher,
        }
    }

    fn songs_ref(&self) -> Option<impl Deref<Target = Vec<SongDescription>> + '_> {
        self.app_model.map_state_opt(|s| {
            Some(
                &s.browser
                    .playlist_details_state(&self.id)?
                    .content
                    .as_ref()?
                    .songs,
            )
        })
    }

    pub fn get_playlist_info(&self) -> Option<impl Deref<Target = PlaylistDescription> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser.playlist_details_state(&self.id)?.content.as_ref())
    }

    pub fn load_playlist_info(&self) {
        let api = self.app_model.get_spotify();
        let id = self.id.clone();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_playlist(&id).await {
                Ok(playlist) => Some(BrowserAction::SetPlaylistDetails(playlist).into()),
                Err(err) => handle_error(err),
            }
        }));
    }
}

impl PlaylistDetailsModel {
    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }
}

impl PlaylistModel for PlaylistDetailsModel {
    fn current_song_id(&self) -> Option<String> {
        self.state().playback.current_song_id.clone()
    }

    fn songs(&self) -> Vec<SongModel> {
        let songs = self.songs_ref();
        match songs {
            Some(songs) => songs
                .iter()
                .enumerate()
                .map(|(i, s)| s.to_song_model(i))
                .collect(),
            None => vec![],
        }
    }

    fn play_song(&self, id: String) {
        let source = PlaylistSource::Playlist(self.id.clone());
        if self.app_model.get_state().playback.source != source {
            let songs = self.songs_ref();
            if let Some(songs) = songs {
                self.dispatcher
                    .dispatch(PlaybackAction::LoadPlaylist(source, songs.clone()).into());
            }
        }
        self.dispatcher.dispatch(PlaybackAction::Load(id).into());
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::PlaylistDetailsLoaded(id)) if id == &self.id
        )
    }

    fn actions_for(&self, id: String) -> Option<gio::ActionGroup> {
        let songs = self.songs_ref()?;
        let song = songs.iter().find(|song| song.id == id)?;

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
        let copy_uri = SimpleAction::new("copy_uri", None);
        copy_uri.connect_activate(move |_, _| {
            let clipboard = Clipboard::get(&SELECTION_CLIPBOARD);
            clipboard.set_text(&format!("spotify:track:{}", &track_id));
        });
        group.add_action(&copy_uri);

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
        let songs = self.songs_ref()?;
        let song = songs.iter().find(|song| song.id == id)?;

        let menu = gio::Menu::new();
        menu.append(Some("View album"), Some("song.view_album"));
        for (i, artist) in song.artists.iter().enumerate() {
            menu.append(
                Some(&format!("More from {}", artist.name)),
                Some(&format!("song.view_artist_{}", i)),
            );
        }

        menu.append(Some("Copy URI"), Some("song.copy_uri"));
        menu.append(Some("Copy link"), Some("song.copy_link"));

        Some(menu.upcast())
    }
}
