use gio::prelude::*;
use gio::{ActionMapExt, SimpleActionGroup};
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{
    handle_error, labels, PlaylistModel, SelectionTool, SelectionToolsModel,
};
use crate::app::models::*;
use crate::app::state::{
    BrowserAction, BrowserEvent, PlaybackAction, PlaylistSource, SelectionAction, SelectionState,
};
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
        let source = PlaylistSource::Playlist(self.id.clone());
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
            AppEvent::BrowserEvent(BrowserEvent::PlaylistDetailsLoaded(id)) if id == &self.id
        )
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
    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        Some(Box::new(self.app_model.map_state(|s| &s.selection)))
    }

    fn handle_tool_activated(&self, selection: &SelectionState, tool: &SelectionTool) {
        let action = match (tool, tool.default_action()) {
            (_, Some(action)) => Some(action),
            (SelectionTool::SelectAll, None) => self.songs_ref().map(|songs| {
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
