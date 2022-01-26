use gio::prelude::*;
use gio::SimpleActionGroup;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::SimpleHeaderBarModel;
use crate::app::components::{labels, PlaylistModel};
use crate::app::models::*;
use crate::app::state::SelectionContext;
use crate::app::state::{BrowserAction, PlaybackAction, SelectionAction, SelectionState};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, BatchQuery, SongsSource};

pub struct PlaylistDetailsModel {
    pub id: String,
    _editable_selection_context: SelectionContext,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl PlaylistDetailsModel {
    pub fn new(id: String, app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            id: id.clone(),
            _editable_selection_context: SelectionContext::EditablePlaylist(id),
            app_model,
            dispatcher,
        }
    }

    fn is_playlist_editable(&self) -> bool {
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

    pub fn load_playlist_info(&self) {
        let api = self.app_model.get_spotify();
        let id = self.id.clone();
        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.get_playlist(&id)
                    .await
                    .map(|playlist| BrowserAction::SetPlaylistDetails(Box::new(playlist)).into())
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
        let loader = self.app_model.get_batch_loader();

        self.dispatcher.dispatch_async(Box::pin(async move {
            let action = loader
                .query(next_query, |song_batch| {
                    BrowserAction::AppendPlaylistTracks(id, Box::new(song_batch)).into()
                })
                .await;
            Some(action)
        }));

        Some(())
    }

    pub fn view_owner(&self) {
        if let Some(playlist) = self.get_playlist_info() {
            let owner = &playlist.owner.id;
            self.dispatcher
                .dispatch(AppAction::ViewUser(owner.to_owned()));
        }
    }
}

impl PlaylistModel for PlaylistDetailsModel {
    fn song_list_model(&self) -> SongListModel {
        self.app_model
            .get_state()
            .browser
            .playlist_details_state(&self.id)
            .expect("illegal attempt to read playlist_details_state")
            .songs
            .clone()
    }

    fn current_song_id(&self) -> Option<String> {
        self.app_model.get_state().playback.current_song_id()
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
        self.dispatcher.dispatch(AppAction::EnableSelection(
            self.selection_context().unwrap().clone(),
        ));
        true
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        Some(Box::new(self.app_model.map_state(|s| &s.selection)))
    }
}

impl SimpleHeaderBarModel for PlaylistDetailsModel {
    fn title(&self) -> Option<String> {
        None
    }

    fn title_updated(&self, _: &AppEvent) -> bool {
        false
    }

    fn selection_context(&self) -> Option<&SelectionContext> {
        Some(if self.is_playlist_editable() {
            &self._editable_selection_context
        } else {
            &SelectionContext::Playlist
        })
    }

    fn select_all(&self) {
        let songs: Vec<SongDescription> = self.song_list_model().collect();
        self.dispatcher
            .dispatch(SelectionAction::Select(songs).into());
    }
}
