use gio::prelude::*;
use gio::SimpleActionGroup;
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::api::SpotifyApiError;
use crate::app::components::labels;
use crate::app::components::HeaderBarModel;
use crate::app::components::PlaylistModel;
use crate::app::components::SimpleHeaderBarModel;
use crate::app::components::SimpleHeaderBarModelWrapper;
use crate::app::dispatch::ActionDispatcher;
use crate::app::models::*;
use crate::app::state::SelectionContext;
use crate::app::state::{BrowserAction, PlaybackAction, SelectionAction, SelectionState};
use crate::app::{AppAction, AppEvent, AppModel, AppState, BatchQuery, SongsSource};

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
                let album = api.get_album(&id).await;
                match album {
                    Ok(album) => Ok(BrowserAction::SetAlbumDetails(Box::new(album)).into()),
                    Err(SpotifyApiError::BadStatus(400, _))
                    | Err(SpotifyApiError::BadStatus(404, _)) => {
                        Ok(BrowserAction::NavigationPop.into())
                    }
                    Err(e) => Err(e),
                }
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

    pub fn toggle_play_album(&self) {
        if let Some(album) = self.get_album_description() {
            if !self.playlist_is_playing() {
                if self.state().playback.is_shuffled() {
                    self.dispatcher
                        .dispatch(AppAction::PlaybackAction(PlaybackAction::ToggleShuffle));
                }
                let id_of_first_song = album.songs.songs[0].id.as_str();
                self.play_song_at(0, id_of_first_song);
                return;
            }
            if self.state().playback.is_playing() {
                self.dispatcher
                    .dispatch(AppAction::PlaybackAction(PlaybackAction::Pause));
            } else {
                self.dispatcher
                    .dispatch(AppAction::PlaybackAction(PlaybackAction::Play));
            }
        }
    }

    pub fn load_more(&self) -> Option<()> {
        let last_batch = self.song_list_model().last_batch()?;
        let query = BatchQuery {
            source: SongsSource::Album(self.id.clone()),
            batch: last_batch,
        };

        let id = self.id.clone();
        let next_query = query.next()?;
        let loader = self.app_model.get_batch_loader();

        self.dispatcher.dispatch_async(Box::pin(async move {
            loader
                .query(next_query, |_s, song_batch| {
                    BrowserAction::AppendAlbumTracks(id, Box::new(song_batch)).into()
                })
                .await
        }));

        Some(())
    }

    pub fn to_headerbar_model(self: &Rc<Self>) -> Rc<impl HeaderBarModel> {
        Rc::new(SimpleHeaderBarModelWrapper::new(
            self.clone(),
            self.app_model.clone(),
            self.dispatcher.box_clone(),
        ))
    }
}

impl PlaylistModel for DetailsModel {
    fn song_list_model(&self) -> SongListModel {
        self.app_model
            .get_state()
            .browser
            .details_state(&self.id)
            .expect("illegal attempt to read details_state")
            .songs
            .clone()
    }

    fn is_paused(&self) -> bool {
        !self.app_model.get_state().playback.is_playing()
    }

    fn show_song_covers(&self) -> bool {
        false
    }

    fn select_song(&self, id: &str) {
        let songs = self.song_list_model();
        if let Some(song) = songs.get(id) {
            self.dispatcher
                .dispatch(SelectionAction::Select(vec![song.description().clone()]).into());
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
        self.state().playback.current_song_id()
    }

    fn is_playing(&self) -> bool {
        self.state().playback.is_playing()
    }

    fn playlist_is_playing(&self) -> bool {
        if let Some(source) = self.state().playback.current_source() {
            if let Some(uri) = source.spotify_uri() {
                if let Some(album) = self.get_album_description() {
                    uri == format!("spotify:album:{}", album.id)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn play_song_at(&self, pos: usize, id: &str) {
        let source = SongsSource::Album(self.id.clone());
        let batch = self.song_list_model().song_batch_for(pos);
        if let Some(batch) = batch {
            self.dispatcher
                .dispatch(PlaybackAction::LoadPagedSongs(source, batch).into());
            self.dispatcher
                .dispatch(PlaybackAction::Load(id.to_string()).into());
        }
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let song = self.song_list_model().get(id)?;
        let song = song.description();

        let group = SimpleActionGroup::new();

        for view_artist in song.make_artist_actions(self.dispatcher.box_clone(), None) {
            group.add_action(&view_artist);
        }
        group.add_action(&song.make_link_action(None));
        group.add_action(&song.make_queue_action(self.dispatcher.box_clone(), None));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let song = self.song_list_model().get(id)?;
        let song = song.description();

        let menu = gio::Menu::new();
        for artist in song.artists.iter() {
            menu.append(
                Some(&labels::more_from_label(&artist.name)),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some(&*labels::COPY_LINK), Some("song.copy_link"));
        menu.append(Some(&*labels::ADD_TO_QUEUE), Some("song.queue"));
        Some(menu.upcast())
    }
}

impl SimpleHeaderBarModel for DetailsModel {
    fn title(&self) -> Option<String> {
        None
    }

    fn title_updated(&self, _: &AppEvent) -> bool {
        false
    }

    fn selection_context(&self) -> Option<SelectionContext> {
        Some(SelectionContext::Default)
    }

    fn select_all(&self) {
        let songs: Vec<SongDescription> = self.song_list_model().collect();
        self.dispatcher
            .dispatch(SelectionAction::Select(songs).into());
    }
}
