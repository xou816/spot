use super::{
    AlbumInfoState, ArtistState, DetailsState, HomeState, PlaylistDetailsState, ScreenName,
    SearchState, UpdatableState, UserState,
};
use crate::api::client::AlbumInfo;
use crate::app::models::*;
use crate::app::state::AppAction;
use std::iter::Iterator;

#[derive(Clone, Debug)]
pub enum BrowserAction {
    SetLibraryContent(Vec<AlbumDescription>),
    AppendLibraryContent(Vec<AlbumDescription>),
    SetPlaylistsContent(Vec<PlaylistDescription>),
    AppendPlaylistsContent(Vec<PlaylistDescription>),
    RemoveTracksFromPlaylist(Vec<String>),
    SetAlbumDetails(AlbumDescription),
    SetAlbumInfo(AlbumInfo),
    SetPlaylistDetails(PlaylistDescription),
    AppendPlaylistTracks(String, SongBatch),
    Search(String),
    SetSearchResults(SearchResults),
    SetArtistDetails(ArtistDescription),
    AppendArtistReleases(Vec<AlbumDescription>),
    NavigationPush(ScreenName),
    NavigationPop,
    NavigationPopTo(ScreenName),
    SaveAlbum(AlbumDescription),
    UnsaveAlbum(String),
    SetUserDetails(UserDescription),
    AppendUserPlaylists(Vec<PlaylistDescription>),
}

impl From<BrowserAction> for AppAction {
    fn from(browser_action: BrowserAction) -> Self {
        Self::BrowserAction(browser_action)
    }
}

impl BrowserAction {
    // additional screen names that should handle this action
    fn additional_targets(&self) -> Vec<ScreenName> {
        match self {
            BrowserAction::SaveAlbum(_) | BrowserAction::UnsaveAlbum(_) => {
                vec![ScreenName::Home]
            }
            _ => vec![],
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BrowserEvent {
    LibraryUpdated,
    SavedPlaylistsUpdated,
    AlbumInfoUpdated,
    AlbumDetailsLoaded(String),
    PlaylistDetailsLoaded(String),
    PlaylistTracksAppended(String, usize),
    PlaylistTracksRemoved(String, Vec<String>),
    SearchUpdated,
    SearchResultsUpdated,
    ArtistDetailsUpdated(String),
    NavigationPushed(ScreenName),
    NavigationPopped,
    NavigationPoppedTo(ScreenName),
    AlbumSaved(String),
    AlbumUnsaved(String),
    UserDetailsUpdated(String),
}

pub enum BrowserScreen {
    Home(HomeState),
    AlbumDetails(DetailsState),
    AlbumInfo(AlbumInfoState),
    Search(SearchState),
    Artist(ArtistState),
    PlaylistDetails(PlaylistDetailsState),
    User(UserState),
}

impl BrowserScreen {
    fn from_name(name: &ScreenName) -> Self {
        match name {
            ScreenName::Home => BrowserScreen::Home(Default::default()),
            ScreenName::AlbumDetails(id) => {
                BrowserScreen::AlbumDetails(DetailsState::new(id.to_string()))
            }
            ScreenName::AlbumInfo(id) => {
                BrowserScreen::AlbumInfo(AlbumInfoState::new(id.to_string()))
            }
            ScreenName::Search => BrowserScreen::Search(Default::default()),
            ScreenName::Artist(id) => BrowserScreen::Artist(ArtistState::new(id.to_string())),
            ScreenName::PlaylistDetails(id) => {
                BrowserScreen::PlaylistDetails(PlaylistDetailsState::new(id.to_string()))
            }
            ScreenName::User(id) => BrowserScreen::User(UserState::new(id.to_string())),
        }
    }

    fn state(&mut self) -> &mut dyn UpdatableState<Action = BrowserAction, Event = BrowserEvent> {
        match self {
            Self::Home(state) => state,
            Self::AlbumDetails(state) => state,
            Self::AlbumInfo(state) => state,
            Self::Search(state) => state,
            Self::Artist(state) => state,
            Self::PlaylistDetails(state) => state,
            Self::User(state) => state,
        }
    }
}

impl NamedScreen for BrowserScreen {
    type Name = ScreenName;

    fn name(&self) -> &Self::Name {
        match self {
            Self::Home(state) => &state.name,
            Self::AlbumDetails(state) => &state.name,
            Self::AlbumInfo(state) => &state.name,
            Self::Search(state) => &state.name,
            Self::Artist(state) => &state.name,
            Self::PlaylistDetails(state) => &state.name,
            Self::User(state) => &state.name,
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

    fn screen_visibility(&self, name: &Screen::Name) -> ScreenState {
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

    pub fn find_screen_mut(&mut self, name: &ScreenName) -> Option<&mut BrowserScreen> {
        self.navigation.iter_mut().find(|s| s.name() == name)
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

    pub fn album_info_state(&self) -> Option<&AlbumInfoState> {
        self.navigation.iter_rev().find_map(|screen| match screen {
            BrowserScreen::AlbumInfo(state) => Some(state),
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

    pub fn user_state(&self, id: &str) -> Option<&UserState> {
        self.navigation.iter_rev().find_map(|screen| match screen {
            BrowserScreen::User(state) if state.id == id => Some(state),
            _ => None,
        })
    }

    fn push_if_needed(&mut self, name: ScreenName) -> Vec<BrowserEvent> {
        let navigation = &mut self.navigation;
        let screen_visibility = navigation.screen_visibility(&name);

        match screen_visibility {
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
            _ => {
                let mut events: Vec<Self::Event> = vec![];
                events.extend(
                    action
                        .additional_targets()
                        .iter()
                        .filter_map(|name| {
                            Some(
                                self.find_screen_mut(name)?
                                    .state()
                                    .update_with(action.clone()),
                            )
                        })
                        .flatten()
                        .collect::<Vec<Self::Event>>(),
                );
                events.extend(self.navigation.current_mut().state().update_with(action));
                events
            }
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
