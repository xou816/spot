use gdk::SELECTION_CLIPBOARD;
use gio::prelude::*;
use gio::{ActionMapExt, SimpleAction, SimpleActionGroup};
use gtk::Clipboard;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{handle_error, PlaylistModel};
use crate::app::models::*;
use crate::app::state::{BrowserAction, BrowserEvent, PlaybackAction, PlaylistSource};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, ListStore};

pub struct ArtistDetailsModel {
    pub id: String,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl ArtistDetailsModel {
    pub fn new(id: String, app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            id,
            app_model,
            dispatcher,
        }
    }

    fn tracks_ref(&self) -> Option<impl Deref<Target = Vec<SongDescription>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.artist_state(&self.id)?.top_tracks))
    }

    pub fn get_artist_name(&self) -> Option<impl Deref<Target = String> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser.artist_state(&self.id)?.artist.as_ref())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumModel>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.artist_state(&self.id)?.albums))
    }

    pub fn load_artist_details(&self, id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_artist(&id[..]).await {
                Ok(artist) => Some(BrowserAction::SetArtistDetails(artist).into()),
                Err(err) => handle_error(err),
            }
        }));
    }

    pub fn open_album(&self, id: &str) {
        self.dispatcher
            .dispatch(AppAction::ViewAlbum(id.to_string()));
    }

    pub fn load_more(&self) -> Option<()> {
        let api = self.app_model.get_spotify();
        let state = self.app_model.get_state();
        let next_page = &state.browser.artist_state(&self.id)?.next_page;

        let id = next_page.data.clone();
        let batch_size = next_page.batch_size;
        let offset = next_page.next_offset?;

        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_artist_albums(&id, offset, batch_size).await {
                Ok(albums) => Some(BrowserAction::AppendArtistReleases(albums).into()),
                Err(err) => handle_error(err),
            }
        }));

        Some(())
    }
}

impl PlaylistModel for ArtistDetailsModel {
    fn select_song(&self, id: &str) {}
    fn deselect_song(&self, id: &str) {}

    fn is_selection_enabled(&self) -> bool {
        false
    }

    fn songs(&self) -> Vec<SongModel> {
        let tracks = self.tracks_ref();
        match tracks {
            Some(tracks) => tracks
                .iter()
                .enumerate()
                .map(|(i, s)| s.to_song_model(i))
                .collect(),
            None => vec![],
        }
    }

    fn current_song_id(&self) -> Option<String> {
        self.app_model.get_state().playback.current_song_id.clone()
    }

    fn play_song(&self, id: &str) {
        let tracks = self.tracks_ref();
        if let Some(tracks) = tracks {
            self.dispatcher.dispatch(
                PlaybackAction::LoadPlaylist(PlaylistSource::None, tracks.clone()).into(),
            );
            self.dispatcher
                .dispatch(PlaybackAction::Load(id.to_string()).into());
        }
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::ArtistDetailsUpdated(id)) if id == &self.id
        )
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let songs = self.tracks_ref()?;
        let song = songs.iter().find(|&song| &song.id == id)?;

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
            clipboard.set_text(&format!("https://open.spotify.com/track/{}", &track_id,));
        });
        group.add_action(&copy_link);

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let songs = self.tracks_ref()?;
        let song = songs.iter().find(|&song| &song.id == id)?;

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
