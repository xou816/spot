use gio::prelude::*;
use gio::{ActionMapExt, SimpleActionGroup};
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{handle_error, PlaylistModel};
use crate::app::models::*;
use crate::app::state::{
    BrowserAction, BrowserEvent, PlaybackAction, PlaylistSource, SelectionAction, SelectionState,
};
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
        let song = songs.iter().find(|&song| song.id == id)?;

        let group = SimpleActionGroup::new();

        for view_artist in song.make_artist_actions(self.dispatcher.box_clone(), None) {
            group.add_action(&view_artist);
        }
        group.add_action(&song.make_album_action(self.dispatcher.box_clone(), None));
        group.add_action(&song.make_link_action(None));
        group.add_action(&song.make_queue_action(self.dispatcher.box_clone(), None));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let songs = self.tracks_ref()?;
        let song = songs.iter().find(|&song| song.id == id)?;

        let menu = gio::Menu::new();
        menu.append(Some("View album"), Some("song.view_album"));
        for artist in song.artists.iter().filter(|a| self.id != a.id) {
            menu.append(
                Some(&format!("More from {}", artist.name)),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }
        menu.append(Some("Copy link"), Some("song.copy_link"));
        menu.append(Some("Queue"), Some("song.queue"));
        Some(menu.upcast())
    }

    fn select_song(&self, id: &str) {
        let song = self
            .tracks_ref()
            .and_then(|songs| songs.iter().find(|&song| song.id == id).cloned());
        if let Some(song) = song {
            self.dispatcher
                .dispatch(SelectionAction::Select(song).into());
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
