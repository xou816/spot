use crate::app::models::SongDescription;
use crate::app::state::{AppAction, AppEvent, UpdatableState};

#[derive(Clone, Debug)]
pub enum SelectionAction {
    ChangeSelectionMode(bool),
    Select(SongDescription),
    Deselect(String),
}

impl Into<AppAction> for SelectionAction {
    fn into(self) -> AppAction {
        AppAction::SelectionAction(self)
    }
}

#[derive(Clone, Debug)]
pub enum SelectionEvent {
    SelectionModeChanged(bool),
    Selected(String),
    Deselected(String),
}

impl Into<AppEvent> for SelectionEvent {
    fn into(self) -> AppEvent {
        AppEvent::SelectionEvent(self)
    }
}

pub struct SelectionState {
    selected_songs: Option<Vec<SongDescription>>,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_songs: None,
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
            let selected = selected_songs.iter().find(|&t| &t.id == id).is_some();
            if selected {
                selected_songs.retain(|t| &t.id != id);
            }
            selected
        } else {
            false
        }
    }

    pub fn is_selection_enabled(&self) -> bool {
        self.selected_songs.is_some()
    }

    pub fn is_song_selected(&self, id: &str) -> bool {
        self.selected_songs
            .as_ref()
            .map(|s| s.iter().find(|&t| &t.id == id).is_some())
            .unwrap_or(false)
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
            SelectionAction::ChangeSelectionMode(active) => {
                if self.selected_songs.is_some() != active {
                    if active {
                        self.selected_songs = Some(vec![]);
                        vec![SelectionEvent::SelectionModeChanged(true)]
                    } else {
                        self.selected_songs = None;
                        vec![SelectionEvent::SelectionModeChanged(false)]
                    }
                } else {
                    vec![]
                }
            }
            SelectionAction::Select(track) => {
                let id = track.id.clone();
                if self.select(track) {
                    vec![SelectionEvent::Selected(id)]
                } else {
                    vec![]
                }
            }
            SelectionAction::Deselect(id) => {
                if self.deselect(&id) {
                    vec![SelectionEvent::Deselected(id)]
                } else {
                    vec![]
                }
            }
        }
    }
}
