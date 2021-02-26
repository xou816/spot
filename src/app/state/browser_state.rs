use super::{
    ArtistState, DetailsState, HomeState, PlaylistDetailsState, ScreenName, SearchState,
    UpdatableState,
};
use crate::app::models::*;
use crate::app::state::AppAction;
use std::convert::Into;
use std::iter::Iterator;

#[derive(Clone, Debug)]
pub enum BrowserAction {
    SetLibraryContent(Vec<AlbumDescription>),
    AppendLibraryContent(Vec<AlbumDescription>),
    SetPlaylistsContent(Vec<PlaylistDescription>),
    AppendPlaylistsContent(Vec<PlaylistDescription>),
    SetAlbumDetails(AlbumDescription),
    SetPlaylistDetails(PlaylistDescription),
    Search(String),
    SetSearchResults(SearchResults),
    SetArtistDetails(ArtistDescription),
    AppendArtistReleases(Vec<AlbumDescription>),
    NavigationPush(ScreenName),
    NavigationPop,
    NavigationPopTo(ScreenName),
    SaveAlbum(AlbumDescription),
    UnsaveAlbum(String),
}

impl Into<AppAction> for BrowserAction {
    fn into(self) -> AppAction {
        AppAction::BrowserAction(self)
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BrowserEvent {
    LibraryUpdated,
    SavedPlaylistsUpdated,
    AlbumDetailsLoaded(String),
    PlaylistDetailsLoaded(String),
    SearchUpdated,
    SearchResultsUpdated,
    ArtistDetailsUpdated(String),
    NavigationPushed(ScreenName),
    NavigationPopped,
    NavigationPoppedTo(ScreenName),
    AlbumSaved(String),
    AlbumUnsaved(String),
}

pub enum BrowserScreen {
    Home(HomeState),
    AlbumDetails(DetailsState),
    Search(SearchState),
    Artist(ArtistState),
    PlaylistDetails(PlaylistDetailsState),
}

impl BrowserScreen {
    fn from_name(name: &ScreenName) -> Self {
        match name {
            ScreenName::Home => BrowserScreen::Home(Default::default()),
            ScreenName::AlbumDetails(id) => {
                BrowserScreen::AlbumDetails(DetailsState::new(id.to_string()))
            }
            ScreenName::Search => BrowserScreen::Search(Default::default()),
            ScreenName::Artist(id) => BrowserScreen::Artist(ArtistState::new(id.to_string())),
            ScreenName::PlaylistDetails(id) => {
                BrowserScreen::PlaylistDetails(PlaylistDetailsState::new(id.to_string()))
            }
        }
    }

    fn state(&mut self) -> &mut dyn UpdatableState<Action = BrowserAction, Event = BrowserEvent> {
        match self {
            Self::Home(state) => state,
            Self::AlbumDetails(state) => state,
            Self::Search(state) => state,
            Self::Artist(state) => state,
            Self::PlaylistDetails(state) => state,
        }
    }
}

impl NamedScreen for BrowserScreen {
    type Name = ScreenName;

    fn name(&self) -> &Self::Name {
        match self {
            Self::Home(state) => &state.name,
            Self::AlbumDetails(state) => &state.name,
            Self::Search(state) => &state.name,
            Self::Artist(state) => &state.name,
            Self::PlaylistDetails(state) => &state.name,
        }
    }
}

pub trait NamedScreen {
    type Name: PartialEq;
    fn name(&self) -> &Self::Name;
}

#[derive(Debug)]
enum ScreenState {
    NotPresent,
    Present,
    Current,
}

struct NavStack<Screen>(Vec<Screen>);

impl<Screen> NavStack<Screen>
where
    Screen: NamedScreen,
{
    fn new(initial: Screen) -> Self {
        Self(vec![initial])
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Screen> {
        self.0.iter_mut()
    }

    fn iter_rev(&self) -> impl Iterator<Item = &Screen> {
        self.0.iter().rev()
    }

    fn count(&self) -> usize {
        self.0.len()
    }

    fn current(&self) -> &Screen {
        self.0.last().unwrap()
    }

    fn current_mut(&mut self) -> &mut Screen {
        self.0.last_mut().unwrap()
    }

    fn can_pop(&self) -> bool {
        self.0.len() > 1
    }

    fn push(&mut self, screen: Screen) {
        self.0.push(screen)
    }

    fn pop(&mut self) -> bool {
        if self.can_pop() {
            self.0.pop();
            true
        } else {
            false
        }
    }

    fn pop_to(&mut self, name: &Screen::Name) {
        let split = self.0.iter().position(|s| s.name() == name).unwrap();
        self.0.truncate(split + 1);
    }

    fn screen_state(&self, name: &Screen::Name) -> ScreenState {
        self.0
            .iter()
            .rev()
            .enumerate()
            .find_map(|(i, screen)| {
                let is_screen = screen.name() == name;
                match (i, is_screen) {
                    (0, true) => Some(ScreenState::Current),
                    (_, true) => Some(ScreenState::Present),
                    (_, _) => None,
                }
            })
            .unwrap_or(ScreenState::NotPresent)
    }
}

pub struct BrowserState {
    navigation: NavStack<BrowserScreen>,
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            navigation: NavStack::new(BrowserScreen::Home(Default::default())),
        }
    }

    pub fn current_screen(&self) -> &ScreenName {
        self.navigation.current().name()
    }

    pub fn can_pop(&self) -> bool {
        self.navigation.can_pop()
    }

    pub fn count(&self) -> usize {
        self.navigation.count()
    }

    pub fn home_state(&self) -> Option<&HomeState> {
        self.navigation.iter_rev().find_map(|screen| match screen {
            BrowserScreen::Home(state) => Some(state),
            _ => None,
        })
    }

    pub fn details_state(&self, id: &str) -> Option<&DetailsState> {
        self.navigation.iter_rev().find_map(|screen| match screen {
            BrowserScreen::AlbumDetails(state) if state.id == id => Some(state),
            _ => None,
        })
    }

    pub fn search_state(&self) -> Option<&SearchState> {
        self.navigation.iter_rev().find_map(|screen| match screen {
            BrowserScreen::Search(state) => Some(state),
            _ => None,
        })
    }

    pub fn artist_state(&self, id: &str) -> Option<&ArtistState> {
        self.navigation.iter_rev().find_map(|screen| match screen {
            BrowserScreen::Artist(state) if state.id == id => Some(state),
            _ => None,
        })
    }

    pub fn playlist_details_state(&self, id: &str) -> Option<&PlaylistDetailsState> {
        self.navigation.iter_rev().find_map(|screen| match screen {
            BrowserScreen::PlaylistDetails(state) if state.id == id => Some(state),
            _ => None,
        })
    }

    fn push_if_needed(&mut self, name: ScreenName) -> Vec<BrowserEvent> {
        let navigation = &mut self.navigation;
        let screen_state = navigation.screen_state(&name);

        match screen_state {
            ScreenState::Current => vec![],
            ScreenState::Present => {
                navigation.pop_to(&name);
                vec![BrowserEvent::NavigationPoppedTo(name)]
            }
            ScreenState::NotPresent => {
                navigation.push(BrowserScreen::from_name(&name));
                vec![BrowserEvent::NavigationPushed(name)]
            }
        }
    }
}

impl UpdatableState for BrowserState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        let can_pop = self.navigation.can_pop();

        match action {
            BrowserAction::Search(_) => {
                let mut events = self.push_if_needed(ScreenName::Search);

                let mut update_events = self.navigation.current_mut().state().update_with(action);
                events.append(&mut update_events);
                events
            }
            BrowserAction::NavigationPush(name) => self.push_if_needed(name),
            BrowserAction::NavigationPopTo(name) => {
                self.navigation.pop_to(&name);
                vec![BrowserEvent::NavigationPoppedTo(name)]
            }
            BrowserAction::NavigationPop if can_pop => {
                self.navigation.pop();
                vec![BrowserEvent::NavigationPopped]
            }
            _ => self
                .navigation
                .iter_mut()
                .map(|screen| screen.state().update_with(action.clone()))
                .flatten()
                .collect(),
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_navigation_push() {
        let mut state = BrowserState::new();

        assert_eq!(*state.current_screen(), ScreenName::Home);
        assert_eq!(state.count(), 1);

        let new_screen = ScreenName::Artist("some_id".to_string());
        state.update_with(BrowserAction::NavigationPush(new_screen.clone()));

        assert_eq!(state.current_screen(), &new_screen);
        assert_eq!(state.count(), 2);
        assert_eq!(state.artist_state("some_id").is_some(), true);
    }

    #[test]
    fn test_navigation_pop() {
        let mut state = BrowserState::new();
        let new_screen = ScreenName::Artist("some_id".to_string());
        state.update_with(BrowserAction::NavigationPush(new_screen.clone()));

        assert_eq!(state.current_screen(), &new_screen);
        assert_eq!(state.count(), 2);

        state.update_with(BrowserAction::NavigationPop);
        assert_eq!(state.current_screen(), &ScreenName::Home);
        assert_eq!(state.count(), 1);

        let events = state.update_with(BrowserAction::NavigationPop);
        assert_eq!(state.current_screen(), &ScreenName::Home);
        assert_eq!(state.count(), 1);
        assert_eq!(events, vec![]);
    }

    #[test]
    fn test_navigation_push_same_screen() {
        let mut state = BrowserState::new();
        let new_screen = ScreenName::Artist("some_id".to_string());
        state.update_with(BrowserAction::NavigationPush(new_screen.clone()));

        assert_eq!(state.current_screen(), &new_screen);
        assert_eq!(state.count(), 2);

        let events = state.update_with(BrowserAction::NavigationPush(new_screen.clone()));
        assert_eq!(state.current_screen(), &new_screen);
        assert_eq!(state.count(), 2);
        assert_eq!(events, vec![]);
    }

    #[test]
    fn test_navigation_push_same_screen_will_pop() {
        let mut state = BrowserState::new();
        let new_screen = ScreenName::Artist("some_id".to_string());
        state.update_with(BrowserAction::NavigationPush(new_screen.clone()));
        state.update_with(BrowserAction::NavigationPush(ScreenName::Search));

        assert_eq!(state.current_screen(), &ScreenName::Search);
        assert_eq!(state.count(), 3);

        let events = state.update_with(BrowserAction::NavigationPush(new_screen.clone()));
        assert_eq!(state.current_screen(), &new_screen);
        assert_eq!(state.count(), 2);
        assert_eq!(events, vec![BrowserEvent::NavigationPoppedTo(new_screen)]);
    }
}
