use gettextrs::gettext;
use gio::prelude::*;
use gio::SimpleActionGroup;
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::api::SpotifyApiError;
use crate::app::components::{labels, PlaylistModel};
use crate::app::models::*;
use crate::app::state::SelectionContext;
use crate::app::state::{BrowserAction, PlaybackAction, SelectionAction, SelectionState};
use crate::app::AppState;
use crate::app::{ActionDispatcher, AppAction, AppModel, BatchQuery, SongsSource};

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

    pub fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }

    pub fn is_playlist_editable(&self) -> bool {
        let state = self.app_model.get_state();
        state.logged_user.playlists.iter().any(|p| p.id == self.id)
    }

    pub fn get_playlist_info(&self) -> Option<impl Deref<Target = PlaylistDescription> + '_> {
        self.app_model.map_state_opt(|s| {
            s.browser
                .playlist_details_state(&self.id)?
                .playlist
                .as_ref()
        })
    }

    pub fn is_playing(&self) -> bool {
        self.state().playback.is_playing()
    }

    pub fn playlist_is_playing(&self) -> bool {
        matches!(
            self.app_model.get_state().playback.current_source(),
            Some(SongsSource::Playlist(ref id)) if id == &self.id)
    }

    pub fn toggle_play_playlist(&self) {
        if let Some(playlist) = self.get_playlist_info() {
            if !self.playlist_is_playing() {
                if self.state().playback.is_shuffled() {
                    self.dispatcher
                        .dispatch(AppAction::PlaybackAction(PlaybackAction::ToggleShuffle));
                }
                // The playlist has no songs and the user has still decided to click the play button,
                // lets just do an early return and show an error...
                if playlist.songs.songs.is_empty() {
                    error!("Unable to start playback because songs is empty");
                    self.dispatcher
                        .dispatch(
                        AppAction::ShowNotification(gettext(
                            "An error occured. Check logs for details!",
                        )));
                    return;
                }

                let id_of_first_song = playlist.songs.songs[0].id.as_str();
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

    pub fn load_playlist_info(&self) {
        let api = self.app_model.get_spotify();
        let id = self.id.clone();
        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                let playlist = api.get_playlist(&id).await;
                let playlist_tracks = api.get_playlist_tracks(&id, 0, 100).await?;
                match playlist {
                    Ok(playlist) => {
                        Ok(BrowserAction::SetPlaylistDetails(Box::new(playlist), Box::new(playlist_tracks)).into())
                    }
                    Err(SpotifyApiError::BadStatus(400, _))
                    | Err(SpotifyApiError::BadStatus(404, _)) => {
                        Ok(BrowserAction::NavigationPop.into())
                    }
                    Err(e) => Err(e),
                }
            });
    }

    pub fn load_more_tracks(&self) -> Option<()> {
        let last_batch = self.song_list_model().last_batch()?;
        let query = BatchQuery {
            source: SongsSource::Playlist(self.id.clone()),
            batch: last_batch,
        };

        let id = self.id.clone();
        let next_query = query.next()?;
        debug!("next_query = {:?}", &next_query);
        let loader = self.app_model.get_batch_loader();

        self.dispatcher.dispatch_async(Box::pin(async move {
            loader
                .query(next_query, |_s, song_batch| {
                    BrowserAction::AppendPlaylistTracks(id, Box::new(song_batch)).into()
                })
                .await
        }));

        Some(())
    }

    pub fn update_playlist_details(&self, title: String) {
        let api = self.app_model.get_spotify();
        let id = self.id.clone();
        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                let playlist = api.update_playlist_details(&id, title.clone()).await;
                match playlist {
                    Ok(_) => Ok(AppAction::UpdatePlaylistName(PlaylistSummary { id, title })),
                    Err(e) => Err(e),
                }
            });
    }

    pub fn view_owner(&self) {
        if let Some(playlist) = self.get_playlist_info() {
            let owner = &playlist.owner.id;
            self.dispatcher
                .dispatch(AppAction::ViewUser(owner.to_owned()));
        }
    }

    pub fn disable_selection(&self) {
        self.dispatcher.dispatch(AppAction::CancelSelection);
    }

    pub fn go_back(&self) {
        self.dispatcher
            .dispatch(BrowserAction::NavigationPop.into());
    }
}

impl PlaylistModel for PlaylistDetailsModel {
    fn song_list_model(&self) -> SongListModel {
        self.state()
            .browser
            .playlist_details_state(&self.id)
            .expect("illegal attempt to read playlist_details_state")
            .songs
            .clone()
    }

    fn is_paused(&self) -> bool {
        !self.state().playback.is_playing()
    }

    fn current_song_id(&self) -> Option<String> {
        self.state().playback.current_song_id()
    }

    fn play_song_at(&self, pos: usize, id: &str) {
        let source = SongsSource::Playlist(self.id.clone());
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
        group.add_action(&song.make_album_action(self.dispatcher.box_clone(), None));
        group.add_action(&song.make_link_action(None));
        group.add_action(&song.make_queue_action(self.dispatcher.box_clone(), None));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let song = self.song_list_model().get(id)?;
        let song = song.description();

        let menu = gio::Menu::new();
        menu.append(Some(&*labels::VIEW_ALBUM), Some("song.view_album"));
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

    fn select_song(&self, id: &str) {
        let song = self.song_list_model().get(id);
        if let Some(song) = song {
            self.dispatcher
                .dispatch(SelectionAction::Select(vec![song.into_description()]).into());
        }
    }

    fn deselect_song(&self, id: &str) {
        self.dispatcher
            .dispatch(SelectionAction::Deselect(vec![id.to_string()]).into());
    }

    fn enable_selection(&self) -> bool {
        self.dispatcher
            .dispatch(AppAction::EnableSelection(if self.is_playlist_editable() {
                SelectionContext::EditablePlaylist(self.id.clone())
            } else {
                SelectionContext::Playlist
            }));
        true
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        Some(Box::new(self.app_model.map_state(|s| &s.selection)))
    }
}
