use std::borrow::Cow;
use std::collections::HashSet;

use crate::app::models::SongDescription;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum SelectionAction {
    Select(Vec<SongDescription>),
    Deselect(Vec<String>),
    Clear,
}

impl From<SelectionAction> for AppAction {
    fn from(selection_action: SelectionAction) -> Self {
        Self::SelectionAction(selection_action)
    }
}

#[derive(Clone, Debug)]
pub enum SelectionEvent {
    // Mode means selection active or not
    SelectionModeChanged(bool),
    SelectionChanged,
}

impl From<SelectionEvent> for AppEvent {
    fn from(selection_event: SelectionEvent) -> Self {
        Self::SelectionEvent(selection_event)
    }
}

#[derive(Debug, Clone)]
pub enum SelectionContext {
    ReadOnlyQueue,
    Queue,
    Playlist,
    EditablePlaylist(String),
    SavedTracks,
    Default,
}

pub struct SelectionState {
    selected_songs: Vec<SongDescription>,
    selected_songs_ids: HashSet<String>,
    selection_active: bool,
    pub context: SelectionContext,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_songs: Default::default(),
            selected_songs_ids: Default::default(),
            selection_active: false,
            context: SelectionContext::Default,
        }
    }
}

impl SelectionState {
    fn select(&mut self, song: SongDescription) -> bool {
        let selected = self.selected_songs_ids.contains(&song.id);
        if !selected {
            self.selected_songs_ids.insert(song.id.clone());
            self.selected_songs.push(song);
        }
        !selected
    }

    fn deselect(&mut self, id: &str) -> bool {
        let songs: Vec<SongDescription> = std::mem::take(&mut self.selected_songs)
            .into_iter()
            .filter(|s| s.id != id)
            .collect();
        self.selected_songs = songs;
        self.selected_songs_ids.remove(id)
    }

    pub fn set_mode(&mut self, context: Option<SelectionContext>) -> Option<bool> {
        let currently_active = self.selection_active;
        match (currently_active, context) {
            (false, Some(context)) => {
                *self = Default::default();
                self.selection_active = true;
                self.context = context;
                Some(true)
            }
            (true, None) => {
                *self = Default::default();
                self.selection_active = false;
                Some(false)
            }
            _ => None,
        }
    }

    pub fn is_selection_enabled(&self) -> bool {
        self.selection_active
    }

    pub fn is_song_selected(&self, id: &str) -> bool {
        self.selected_songs_ids.contains(id)
    }

    pub fn count(&self) -> usize {
        self.selected_songs_ids.len()
    }

    // Clears (!) the selection, returns associated memory
    pub fn take_selection(&mut self) -> Vec<SongDescription> {
        std::mem::take(self).selected_songs
    }

    // Just have a look at the selection without changing it
    pub fn peek_selection(&self) -> impl Iterator<Item = &'_ SongDescription> {
        self.selected_songs.iter()
    }
}

impl UpdatableState for SelectionState {
    type Action = SelectionAction;
    type Event = SelectionEvent;

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.into_owned() {
            SelectionAction::Select(tracks) => {
                let changed = tracks
                    .into_iter()
                    .fold(false, |result, track| self.select(track) || result);
                if changed {
                    vec![SelectionEvent::SelectionChanged]
                } else {
                    vec![]
                }
            }
            SelectionAction::Deselect(ids) => {
                let changed = ids
                    .iter()
                    .fold(false, |result, id| self.deselect(id) || result);
                if changed {
                    vec![SelectionEvent::SelectionChanged]
                } else {
                    vec![]
                }
            }
            SelectionAction::Clear => {
                self.take_selection();
                vec![SelectionEvent::SelectionModeChanged(false)]
            }
        }
    }
}
