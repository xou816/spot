use gio::prelude::*;
use gio::{ActionMapExt, SimpleActionGroup};
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{handle_error, PlaylistModel};
use crate::app::dispatch::ActionDispatcher;
use crate::app::models::*;
use crate::app::state::{
    BrowserAction, BrowserEvent, PlaybackAction, PlaylistSource, SelectionAction, SelectionState,
};
use crate::app::{AppAction, AppEvent, AppModel, AppState};

pub struct DetailsModel {
    pub id: String,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl DetailsModel {
    pub fn new(id: String, app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            id,
            app_model,
            dispatcher,
        }
    }

    fn songs_ref(&self) -> Option<impl Deref<Target = Vec<SongDescription>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.details_state(&self.id)?.content.as_ref()?.songs))
    }

    pub fn get_album_info(&self) -> Option<impl Deref<Target = AlbumDescription> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser.details_state(&self.id)?.content.as_ref())
    }

    pub fn load_album_info(&self) {
        let id = self.id.clone();
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match api.get_album(&id).await {
                Ok(album) => Some(BrowserAction::SetAlbumDetails(album).into()),
                Err(err) => handle_error(err),
            }
        }));
    }

    pub fn view_artist(&self) {
        if let Some(album) = self.get_album_info() {
            let artist = &album.artists.first().unwrap().id;
            self.dispatcher
                .dispatch(AppAction::ViewArtist(artist.to_owned()));
        }
    }

    pub fn toggle_save_album(&self) {
        if let Some(album) = self.get_album_info() {
            let id = album.id.clone();
            let is_liked = album.is_liked;

            let api = self.app_model.get_spotify();

            self.dispatcher.dispatch_async(Box::pin(async move {
                if !is_liked {
                    match api.save_album(&id).await {
                        Ok(album) => Some(BrowserAction::SaveAlbum(album).into()),
                        Err(err) => handle_error(err),
                    }
                } else {
                    match api.remove_saved_album(&id).await {
                        Ok(_) => Some(BrowserAction::UnsaveAlbum(id).into()),
                        Err(err) => handle_error(err),
                    }
                }
            }));
        }
    }
}

impl DetailsModel {
    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }
}

impl PlaylistModel for DetailsModel {
    fn select_song(&self, id: &str) {
        let song = self
            .songs_ref()
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

    fn play_song(&self, id: &str) {
        let source = PlaylistSource::Album(self.id.clone());
        if self.app_model.get_state().playback.source != source {
            let songs = self.songs_ref();
            if let Some(songs) = songs {
                self.dispatcher
                    .dispatch(PlaybackAction::LoadPlaylist(source, songs.clone()).into());
            }
        }
        self.dispatcher
            .dispatch(PlaybackAction::Load(id.to_string()).into());
    }

    fn should_refresh_songs(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::AlbumDetailsLoaded(id)) if id == &self.id
        )
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let songs = self.songs_ref()?;
        let song = songs.iter().find(|&song| song.id == id)?;

        let group = SimpleActionGroup::new();

        for view_artist in song.make_artist_actions(self.dispatcher.box_clone(), None) {
            group.add_action(&view_artist);
        }
        group.add_action(&song.make_link_action(None));
        group.add_action(&song.make_queue_action(self.dispatcher.box_clone(), None));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let songs = self.songs_ref()?;
        let song = songs.iter().find(|&song| song.id == id)?;

        let menu = gio::Menu::new();
        for artist in song.artists.iter() {
            menu.append(
                Some(&format!("More from {}", artist.name)),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some("Copy link"), Some("song.copy_link"));
        menu.append(Some("Queue"), Some("song.queue"));

        Some(menu.upcast())
    }
}
