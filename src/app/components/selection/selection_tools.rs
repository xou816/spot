use std::{ops::Deref, sync::Arc};

use crate::app::models::SongDescription;
use crate::app::state::SelectionState;
use crate::app::ActionDispatcher;
use crate::app::{models::PlaylistSummary, state::SelectionAction};
use crate::{api::SpotifyApiClient, app::AppAction};

#[derive(Debug, Clone, Copy)]
pub enum SimpleSelectionTool {
    MoveUp,
    MoveDown,
    Remove,
    SelectAll,
}

#[derive(Debug, Clone)]
pub enum AddSelectionTool {
    AddToQueue,
    AddToPlaylist(PlaylistSummary),
}

#[derive(Debug, Clone)]
pub enum SelectionTool {
    Add(AddSelectionTool),
    Simple(SimpleSelectionTool),
}

pub trait SelectionToolsModel {
    // dependencies
    fn dispatcher(&self) -> Box<dyn ActionDispatcher>;
    fn spotify_client(&self) -> Arc<dyn SpotifyApiClient + Send + Sync>;

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>>;
    fn enabled_selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        self.selection().filter(|s| s.is_selection_enabled())
    }

    fn tools_visible(&self, selection: &SelectionState) -> Vec<SelectionTool>;

    fn handle_tool_activated(&self, selection: &SelectionState, tool: &SelectionTool) {
        self.default_handle_tool_activated(selection, tool)
    }

    fn default_handle_tool_activated(&self, selection: &SelectionState, tool: &SelectionTool) {
        match tool {
            SelectionTool::Add(AddSelectionTool::AddToPlaylist(playlist)) => {
                self.handle_add_to_playlist_tool(selection, &playlist.id);
            }
            SelectionTool::Add(AddSelectionTool::AddToQueue) => {
                self.dispatcher().dispatch(AppAction::QueueSelection);
            }
            _ => {}
        }
    }

    // common tools implementations

    fn handle_select_all_tool<'a>(&self, selection: &SelectionState, songs: &'a [SongDescription]) {
        let all_selected = selection.all_selected(songs.iter().map(|s| &s.id));
        let action = if all_selected {
            SelectionAction::Deselect(songs.iter().map(|s| &s.id).cloned().collect())
        } else {
            SelectionAction::Select(songs.to_vec())
        };
        self.dispatcher().dispatch(action.into());
    }

    fn handle_select_all_tool_borrowed<'a>(
        &self,
        selection: &SelectionState,
        songs: &'a [&'a SongDescription],
    ) {
        let all_selected = selection.all_selected(songs.iter().map(|s| &s.id));
        let action = if all_selected {
            SelectionAction::Deselect(songs.iter().map(|s| &s.id).cloned().collect())
        } else {
            SelectionAction::Select(songs.iter().map(|&s| s.clone()).collect())
        };
        self.dispatcher().dispatch(action.into());
    }

    fn handle_add_to_playlist_tool(&self, selection: &SelectionState, playlist: &str) {
        let api = self.spotify_client();
        let id = playlist.to_string();
        let uris: Vec<String> = selection
            .peek_selection()
            .iter()
            .map(|s| &s.uri)
            .cloned()
            .collect();
        self.dispatcher()
            .call_spotify_and_dispatch(move || async move {
                api.add_to_playlist(&id, uris).await?;
                Ok(SelectionAction::Clear.into())
            })
    }
}
