use super::{
    ArtistState, DetailsState, HomeState, PlaylistDetailsState, ScreenName, SearchState,
    UpdatableState, UserState,
};
use crate::app::models::*;
use crate::app::state::AppAction;
use std::iter::Iterator;

#[derive(Clone, Debug)]
pub enum BrowserAction {
    SetNavigationHidden(bool),
    SetLibraryContent(Vec<AlbumDescription>),
    AppendLibraryContent(Vec<AlbumDescription>),
    SetPlaylistsContent(Vec<PlaylistDescription>),
    AppendPlaylistsContent(Vec<PlaylistDescription>),
    RemoveTracksFromPlaylist(Vec<String>),
    SetAlbumDetails(Box<AlbumFullDescription>),
    AppendAlbumTracks(String, Box<SongBatch>),
    SetPlaylistDetails(Box<PlaylistDescription>),
    AppendPlaylistTracks(String, Box<SongBatch>),
    Search(String),
    SetSearchResults(Box<SearchResults>),
    SetArtistDetails(Box<ArtistDescription>),
    AppendArtistReleases(Vec<AlbumDescription>),
    NavigationPush(ScreenName),
    NavigationPop,
    NavigationPopTo(ScreenName),
    SaveAlbum(Box<AlbumDescription>),
    UnsaveAlbum(String),
    SetUserDetails(Box<UserDescription>),
    AppendUserPlaylists(Vec<PlaylistDescription>),
    AppendSavedTracks(Box<SongBatch>),
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
    NavigationHidden(bool),
    LibraryUpdated,
    SavedPlaylistsUpdated,
    AlbumDetailsLoaded(String),
    AlbumTracksAppended(String),
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
    SavedTracksAppended(usize),
}

pub enum BrowserScreen {
    Home(Box<HomeState>),
    AlbumDetails(Box<DetailsState>),
    Search(Box<SearchState>),
    Artist(Box<ArtistState>),
    PlaylistDetails(Box<PlaylistDetailsState>),
    User(Box<UserState>),
}

impl BrowserScreen {
    fn from_name(name: &ScreenName) -> Self {
        match name {
            ScreenName::Home => BrowserScreen::Home(Default::default()),
            ScreenName::AlbumDetails(id) => {
                BrowserScreen::AlbumDetails(Box::new(DetailsState::new(id.to_string())))
            }
            ScreenName::Search => BrowserScreen::Search(Default::default()),
            ScreenName::Artist(id) => {
                BrowserScreen::Artist(Box::new(ArtistState::new(id.to_string())))
            }
            ScreenName::PlaylistDetails(id) => {
                BrowserScreen::PlaylistDetails(Box::new(PlaylistDetailsState::new(id.to_string())))
            }
            ScreenName::User(id) => BrowserScreen::User(Box::new(UserState::new(id.to_string()))),
        }
    }

    fn state(&mut self) -> &mut dyn UpdatableState<Action = BrowserAction, Event = BrowserEvent> {
        match self {
            Self::Home(state) => &mut **state,
            Self::AlbumDetails(state) => &mut **state,
            Self::Search(state) => &mut **state,
            Self::Artist(state) => &mut **state,
            Self::PlaylistDetails(state) => &mut **state,
            Self::User(state) => &mut **state,
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
    navigation_hidden: bool,
    navigation: NavStack<BrowserScreen>,
}

macro_rules! extract_state {
    ($e:expr, $p:pat if $guard:expr => $i:ident) => {
        extract_state_full!($e, $p if $guard => $i)
    };
    ($e:expr, $p:pat => $i:ident) => {
        extract_state_full!($e, $p if true => $i)
    };
}

macro_rules! extract_state_full {
    ($e:expr, $p:pat if $guard:expr => $i:ident) => {{
        $e.navigation.iter_rev().find_map(|screen| match screen {
            $p if $guard => Some(&**$i),
            _ => None,
        })
    }};
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            navigation_hidden: false,
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
        self.navigation.can_pop() || self.navigation_hidden
    }

    pub fn count(&self) -> usize {
        self.navigation.count()
    }

    pub fn home_state(&self) -> Option<&HomeState> {
        extract_state!(&self, BrowserScreen::Home(s) => s)
    }

    pub fn details_state(&self, id: &str) -> Option<&DetailsState> {
        extract_state!(&self, BrowserScreen::AlbumDetails(state) if state.id == id => state)
    }

    pub fn search_state(&self) -> Option<&SearchState> {
        extract_state!(&self, BrowserScreen::Search(s) => s)
    }

    pub fn artist_state(&self, id: &str) -> Option<&ArtistState> {
        extract_state!(&self, BrowserScreen::Artist(state) if state.id == id => state)
    }

    pub fn playlist_details_state(&self, id: &str) -> Option<&PlaylistDetailsState> {
        extract_state!(&self, BrowserScreen::PlaylistDetails(state) if state.id == id => state)
    }

    pub fn user_state(&self, id: &str) -> Option<&UserState> {
        extract_state!(&self, BrowserScreen::User(state) if state.id == id => state)
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
            BrowserAction::SetNavigationHidden(navigation_hidden) => {
                self.navigation_hidden = navigation_hidden;
                vec![BrowserEvent::NavigationHidden(navigation_hidden)]
            }
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
            BrowserAction::NavigationPop if self.navigation_hidden => {
                self.navigation_hidden = false;
                vec![BrowserEvent::NavigationHidden(false)]
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
