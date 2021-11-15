use gio::prelude::*;
use gio::SimpleActionGroup;
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::labels;
use crate::app::components::PlaylistModel;
use crate::app::components::SimpleScreenModel;
use crate::app::dispatch::ActionDispatcher;
use crate::app::models::*;
use crate::app::state::SelectionContext;
use crate::app::state::{
    BrowserAction, BrowserEvent, PlaybackAction, SelectionAction, SelectionState,
};
use crate::app::{AppAction, AppEvent, AppModel, AppState, BatchQuery, ListDiff, SongsSource};

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

    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }

    fn songs_ref(&self) -> Option<impl Deref<Target = SongList> + '_> {
        self.app_model.map_state_opt(|s| {
            Some(
                &s.browser
                    .details_state(&self.id)?
                    .content
                    .as_ref()?
                    .description
                    .songs,
            )
        })
    }

    pub fn get_album_info(&self) -> Option<impl Deref<Target = AlbumFullDescription> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser.details_state(&self.id)?.content.as_ref())
    }

    pub fn get_album_description(&self) -> Option<impl Deref<Target = AlbumDescription> + '_> {
        self.app_model.map_state_opt(|s| {
            Some(
                &s.browser
                    .details_state(&self.id)?
                    .content
                    .as_ref()?
                    .description,
            )
        })
    }

    pub fn load_album_info(&self) {
        let id = self.id.clone();
        let api = self.app_model.get_spotify();
        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.get_album(&id)
                    .await
                    .map(|album| BrowserAction::SetAlbumDetails(Box::new(album)).into())
            });
    }

    pub fn view_artist(&self) {
        if let Some(album) = self.get_album_description() {
            let artist = &album.artists.first().unwrap().id;
            self.dispatcher
                .dispatch(AppAction::ViewArtist(artist.to_owned()));
        }
    }

    pub fn toggle_save_album(&self) {
        if let Some(album) = self.get_album_description() {
            let id = album.id.clone();
            let is_liked = album.is_liked;

            let api = self.app_model.get_spotify();

            self.dispatcher
                .call_spotify_and_dispatch(move || async move {
                    if !is_liked {
                        api.save_album(&id)
                            .await
                            .map(|album| BrowserAction::SaveAlbum(Box::new(album)).into())
                    } else {
                        api.remove_saved_album(&id)
                            .await
                            .map(|_| BrowserAction::UnsaveAlbum(id).into())
                    }
                });
        }
    }

    pub fn load_more(&self) -> Option<()> {
        let last_batch = self.songs_ref()?.last_batch()?;
        let query = BatchQuery {
            source: SongsSource::Album(self.id.clone()),
            batch: last_batch,
        };

        let id = self.id.clone();
        let next_query = query.next()?;
        let loader = self.app_model.get_batch_loader();

        self.dispatcher.dispatch_async(Box::pin(async move {
            let action = loader
                .query(next_query, |song_batch| {
                    BrowserAction::AppendAlbumTracks(id, Box::new(song_batch)).into()
                })
                .await;
            Some(action)
        }));

        Some(())
    }
}

impl PlaylistModel for DetailsModel {
    fn select_song(&self, id: &str) {
        let song = self.songs_ref().and_then(|songs| songs.get(id).cloned());
        if let Some(song) = song {
            self.dispatcher
                .dispatch(SelectionAction::Select(vec![song]).into());
        }
    }

    fn deselect_song(&self, id: &str) {
        self.dispatcher
            .dispatch(SelectionAction::Deselect(vec![id.to_string()]).into());
    }

    fn enable_selection(&self) -> bool {
        self.dispatcher
            .dispatch(AppAction::EnableSelection(SelectionContext::Default));
        true
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        Some(Box::new(self.app_model.map_state(|s| &s.selection)))
    }

    fn current_song_id(&self) -> Option<String> {
        self.state().playback.current_song_id().cloned()
    }

    fn play_song_at(&self, pos: usize, id: &str) {
        let source = SongsSource::Album(self.id.clone());
        let batch = self.songs_ref().and_then(|songs| songs.song_batch_for(pos));
        if let Some(batch) = batch {
            self.dispatcher
                .dispatch(PlaybackAction::LoadPagedSongs(source, batch).into());
            self.dispatcher
                .dispatch(PlaybackAction::Load(id.to_string()).into());
        }
    }

    fn diff_for_event(&self, event: &AppEvent) -> Option<ListDiff<SongModel>> {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::AlbumDetailsLoaded(id)) if id == &self.id => {
                let songs = self.songs_ref()?;
                Some(ListDiff::Set(songs.iter().map(|s| s.into()).collect()))
            }
            AppEvent::BrowserEvent(BrowserEvent::AlbumTracksAppended(id, index))
                if id == &self.id =>
            {
                let songs = self.songs_ref()?;
                Some(ListDiff::Append(
                    songs.iter().skip(*index).map(|s| s.into()).collect(),
                ))
            }
            _ => None,
        }
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let songs = self.songs_ref()?;
        let song = songs.get(id)?;

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
        let song = songs.get(id)?;

        let menu = gio::Menu::new();
        for artist in song.artists.iter() {
            menu.append(
                Some(&format!(
                    "{} {}",
                    *labels::MORE_FROM,
                    glib::markup_escape_text(&artist.name)
                )),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some(&*labels::COPY_LINK), Some("song.copy_link"));
        menu.append(Some(&*labels::ADD_TO_QUEUE), Some("song.queue"));
        Some(menu.upcast())
    }
}

impl SimpleScreenModel for DetailsModel {
    fn title(&self) -> Option<String> {
        None
    }

    fn title_updated(&self, _: &AppEvent) -> bool {
        false
    }

    fn selection_context(&self) -> Option<&SelectionContext> {
        Some(&SelectionContext::Default)
    }

    fn select_all(&self) {
        if let Some(songs) = self.songs_ref() {
            let songs: Vec<SongDescription> = songs.iter().cloned().collect();
            self.dispatcher
                .dispatch(SelectionAction::Select(songs).into());
        }
    }
}
