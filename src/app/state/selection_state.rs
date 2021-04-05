use crate::app::models::SongDescription;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum SelectionAction {
    Select(Vec<SongDescription>),
    Deselect(Vec<String>),
}

impl Into<AppAction> for SelectionAction {
    fn into(self) -> AppAction {
        AppAction::SelectionAction(self)
    }
}

#[derive(Clone, Debug)]
pub enum SelectionEvent {
    SelectionModeChanged(bool),
    SelectionChanged,
}

impl Into<AppEvent> for SelectionEvent {
    fn into(self) -> AppEvent {
        AppEvent::SelectionEvent(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionContext {
    Queue,
    Global,
}

pub struct SelectionState {
    selected_songs: Option<Vec<SongDescription>>,
    pub context: SelectionContext,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_songs: None,
            context: SelectionContext::Global,
        }
    }
}

impl SelectionState {
    fn select(&mut self, song: SongDescription) -> bool {
        if let Some(selected_songs) = self.selected_songs.as_mut() {
            let not_selected = selected_songs.iter().find(|&t| t.id == song.id).is_none();
            if not_selected {
                selected_songs.push(song);
            }
            not_selected
        } else {
            false
        }
    }

    fn deselect(&mut self, id: &str) -> bool {
        if let Some(selected_songs) = self.selected_songs.as_mut() {
            let selected = selected_songs.iter().any(|t| t.id == id);
            if selected {
                selected_songs.retain(|t| t.id != id);
            }
            selected
        } else {
            false
        }
    }

    pub fn set_mode(&mut self, context: Option<SelectionContext>) -> Option<bool> {
        let currently_active = self.selected_songs.is_some();
        match (currently_active, context) {
            (false, Some(context)) => {
                self.selected_songs = Some(vec![]);
                self.context = context;
                Some(true)
            }
            (true, None) => {
                self.selected_songs = None;
                Some(false)
            }
            _ => None,
        }
    }

    pub fn is_selection_enabled(&self) -> bool {
        self.selected_songs.is_some()
    }

    pub fn is_song_selected(&self, id: &str) -> bool {
        self.selected_songs
            .as_ref()
            .map(|s| s.iter().any(|t| t.id == id))
            .unwrap_or(false)
    }

    pub fn all_selected<'a>(&self, mut ids: impl Iterator<Item = &'a String>) -> bool {
        ids.all(|id| self.is_song_selected(id))
    }

    pub fn count(&self) -> usize {
        self.selected_songs.as_ref().map(|s| s.len()).unwrap_or(0)
    }

    pub fn take_selection(&mut self) -> Vec<SongDescription> {
        self.selected_songs.take().unwrap_or_else(Vec::new)
    }

    pub fn peek_selection(&self) -> &[SongDescription] {
        self.selected_songs.as_ref().map(|s| &s[..]).unwrap_or(&[])
    }
}

impl UpdatableState for SelectionState {
    type Action = SelectionAction;
    type Event = SelectionEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
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
        }
    }
}
