use gio::prelude::*;
use gio::{ActionMapExt, SimpleActionGroup};
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crate::api::SpotifyApiClient;
use crate::app::components::{labels, PlaylistModel, SelectionTool, SelectionToolsModel};
use crate::app::components::{AddSelectionTool, SimpleSelectionTool};
use crate::app::models::*;
use crate::app::state::{
    BrowserAction, BrowserEvent, PlaybackAction, PlaylistSource, SelectionAction, SelectionContext,
    SelectionState,
};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, AppState, ListDiff};

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
                    .playlist
                    .as_ref()?
                    .songs,
            )
        })
    }

    pub fn get_playlist_info(&self) -> Option<impl Deref<Target = PlaylistDescription> + '_> {
        self.app_model.map_state_opt(|s| {
            s.browser
                .playlist_details_state(&self.id)?
                .playlist
                .as_ref()
        })
    }

    fn is_playlist_writable(&self) -> bool {
        let state = self.app_model.get_state();
        state.logged_user.playlists.iter().any(|p| p.id == self.id)
    }

    pub fn load_playlist_info(&self) {
        let api = self.app_model.get_spotify();
        let id = self.id.clone();
        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.get_playlist(&id)
                    .await
                    .map(|playlist| BrowserAction::SetPlaylistDetails(playlist).into())
            });
    }

    pub fn load_more_tracks(&self) -> Option<()> {
        let api = self.app_model.get_spotify();
        let id = self.id.clone();

        let state = self.app_model.get_state();
        let next_batch = &state
            .browser
            .playlist_details_state(&id)?
            .playlist
            .as_ref()?
            .last_batch
            .next()?;
        let next_offset = next_batch.offset;
        let batch_size = next_batch.batch_size;

        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.get_playlist_tracks(&id, next_offset, batch_size)
                    .await
                    .map(|tracks| BrowserAction::AppendPlaylistTracks(id, tracks).into())
            });

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

impl PlaylistDetailsModel {
    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }
}

impl PlaylistModel for PlaylistDetailsModel {
    fn current_song_id(&self) -> Option<String> {
        self.state().playback.current_song_id().cloned()
    }

    fn play_song(&self, id: &str) {
        let source = Some(PlaylistSource::Playlist(self.id.clone()));
        if self.app_model.get_state().playback.source != source {
            if let Some(playlist) = self.get_playlist_info() {
                self.dispatcher.dispatch(
                    PlaybackAction::LoadPagedSongs(
                        source,
                        SongBatch {
                            batch: playlist.last_batch,
                            songs: playlist.songs.clone(),
                        },
                    )
                    .into(),
                );
            }
        }
        self.dispatcher
            .dispatch(PlaybackAction::Load(id.to_string()).into());
    }

    fn diff_for_event(&self, event: &AppEvent) -> Option<ListDiff<SongModel>> {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::PlaylistDetailsLoaded(id))
            | AppEvent::BrowserEvent(BrowserEvent::PlaylistTracksRemoved(id, _))
                if id == &self.id =>
            {
                let songs = self.songs_ref()?;
                Some(ListDiff::Set(
                    songs
                        .iter()
                        .enumerate()
                        .map(|(i, s)| s.to_song_model(i))
                        .collect(),
                ))
            }
            AppEvent::BrowserEvent(BrowserEvent::PlaylistTracksAppended(id, index))
                if id == &self.id =>
            {
                let songs = self.songs_ref()?;
                Some(ListDiff::Append(
                    songs
                        .iter()
                        .enumerate()
                        .skip(*index)
                        .map(|(i, s)| s.to_song_model(i))
                        .collect(),
                ))
            }
            _ => None,
        }
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let songs = self.songs_ref()?;
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
        let songs = self.songs_ref()?;
        let song = songs.iter().find(|&song| song.id == id)?;

        let menu = gio::Menu::new();
        menu.append(Some(&*labels::VIEW_ALBUM), Some("song.view_album"));
        for artist in song.artists.iter() {
            menu.append(
                Some(&format!("{} {}", *labels::MORE_FROM, artist.name)),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some(&*labels::COPY_LINK), Some("song.copy_link"));
        menu.append(Some(&*labels::ADD_TO_QUEUE), Some("song.queue"));

        Some(menu.upcast())
    }

    fn select_song(&self, id: &str) {
        let song = self
            .songs_ref()
            .and_then(|songs| songs.iter().find(|&song| song.id == id).cloned());
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
            .dispatch(AppAction::ChangeSelectionMode(true));
        true
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        Some(Box::new(self.app_model.map_state(|s| &s.selection)))
    }
}

impl SelectionToolsModel for PlaylistDetailsModel {
    fn dispatcher(&self) -> Box<dyn ActionDispatcher> {
        self.dispatcher.box_clone()
    }

    fn spotify_client(&self) -> Arc<dyn SpotifyApiClient + Send + Sync> {
        self.app_model.get_spotify()
    }

    fn tools_visible(&self, _: &SelectionState) -> Vec<SelectionTool> {
        let mut playlists: Vec<SelectionTool> = self
            .app_model
            .get_state()
            .logged_user
            .playlists
            .iter()
            .filter(|p| p.id != self.id)
            .map(|p| SelectionTool::Add(AddSelectionTool::AddToPlaylist(p.clone())))
            .collect();
        let mut tools = vec![
            SelectionTool::Simple(SimpleSelectionTool::SelectAll),
            SelectionTool::Add(AddSelectionTool::AddToQueue),
        ];
        tools.append(&mut playlists);
        if self.is_playlist_writable() {
            tools.push(SelectionTool::Simple(SimpleSelectionTool::Remove));
        }
        tools
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        let selection = self
            .app_model
            .map_state_opt(|s| Some(&s.selection))
            .filter(|s| s.context == SelectionContext::Playlist)?;
        Some(Box::new(selection))
    }

    fn handle_tool_activated(&self, selection: &SelectionState, tool: &SelectionTool) {
        match tool {
            SelectionTool::Simple(SimpleSelectionTool::SelectAll) => {
                if let Some(songs) = self.songs_ref() {
                    self.handle_select_all_tool(selection, &songs[..]);
                }
            }
            SelectionTool::Simple(SimpleSelectionTool::Remove) => {
                self.handle_remove_from_playlist_tool(selection, &self.id);
            }
            _ => self.default_handle_tool_activated(selection, tool),
        };
    }
}
