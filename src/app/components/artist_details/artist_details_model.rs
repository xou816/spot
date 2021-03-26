use gio::prelude::*;
use gio::{ActionMapExt, SimpleActionGroup};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crate::app::components::{labels, PlaylistModel, SelectionTool, SelectionToolsModel};
use crate::app::models::*;
use crate::app::state::{
    BrowserAction, BrowserEvent, PlaybackAction, SelectionAction, SelectionState,
};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, ListDiff, ListStore};

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
        let id = id.clone();
        self.dispatcher.dispatch_spotify_call(move || {
            let api = Arc::clone(&api);
            let id = id.clone();
            async move {
                api.get_artist(&id)
                    .await
                    .map(|artist| BrowserAction::SetArtistDetails(artist).into())
            }
        });
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

        self.dispatcher.dispatch_spotify_call(move || {
            let api = Arc::clone(&api);
            let id = id.clone();
            async move {
                api.get_artist_albums(&id, offset, batch_size)
                    .await
                    .map(|albums| BrowserAction::AppendArtistReleases(albums).into())
            }
        });

        Some(())
    }
}

impl PlaylistModel for ArtistDetailsModel {
    fn current_song_id(&self) -> Option<String> {
        self.app_model.get_state().playback.current_song_id.clone()
    }

    fn play_song(&self, id: &str) {
        let tracks = self.tracks_ref();
        if let Some(tracks) = tracks {
            self.dispatcher
                .dispatch(PlaybackAction::LoadPlaylist(None, tracks.clone()).into());
            self.dispatcher
                .dispatch(PlaybackAction::Load(id.to_string()).into());
        }
    }

    fn diff_for_event(&self, event: &AppEvent) -> Option<ListDiff<SongModel>> {
        if matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::ArtistDetailsUpdated(id)) if id == &self.id
        ) {
            let tracks = self.tracks_ref()?;
            Some(ListDiff::Set(
                tracks
                    .iter()
                    .enumerate()
                    .map(|(i, s)| s.to_song_model(i))
                    .collect(),
            ))
        } else {
            None
        }
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
        menu.append(Some(&*labels::VIEW_ALBUM), Some("song.view_album"));
        for artist in song.artists.iter().filter(|a| self.id != a.id) {
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
            .tracks_ref()
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

impl SelectionToolsModel for ArtistDetailsModel {
    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        Some(Box::new(self.app_model.map_state(|s| &s.selection)))
    }

    fn handle_tool_activated(&self, selection: &SelectionState, tool: &SelectionTool) {
        let action = match (tool, tool.default_action()) {
            (_, Some(action)) => Some(action),
            (SelectionTool::SelectAll, None) => self.tracks_ref().map(|songs| {
                let songs = &*songs;
                let all_selected = selection.all_selected(songs.iter().map(|s| &s.id));

                if all_selected {
                    SelectionAction::Deselect(songs.iter().map(|s| &s.id).cloned().collect())
                } else {
                    SelectionAction::Select(songs.iter().cloned().collect())
                }
                .into()
            }),
            _ => None,
        };
        if let Some(action) = action {
            self.dispatcher.dispatch(action);
        }
    }
}
