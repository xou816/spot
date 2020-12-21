use std::convert::Into;
use std::iter::Iterator;
use crate::app::models::*;
use crate::app::state::AppAction;
use super::{UpdatableState, DetailsState, LibraryState, SearchState, ArtistState, ScreenName};

#[derive(Clone, Debug)]
pub enum BrowserAction {
    SetContent(Vec<AlbumDescription>),
    AppendContent(Vec<AlbumDescription>),
    SetDetails(AlbumDescription),
    Search(String),
    SetSearchResults(Vec<AlbumDescription>),
    NavigationPush(ScreenName),
    NavigationPop,
}

impl Into<AppAction> for BrowserAction {
    fn into(self) -> AppAction {
        AppAction::BrowserAction(self)
    }
}

#[derive(Clone, Debug)]
pub enum BrowserEvent {
    LibraryUpdated,
    DetailsLoaded,
    SearchUpdated,
    SearchResultsUpdated,
    NavigationPushed(ScreenName),
    NavigationPopped,
    NavigationPoppedTo(ScreenName),
}



#[derive(Clone)]
pub enum BrowserScreen {
    Library(LibraryState),
    Details(DetailsState),
    Search(SearchState),
    Artist(ArtistState)
}

impl BrowserScreen {

    fn from_name(name: &ScreenName) -> Self {
        match name {
            ScreenName::Library => BrowserScreen::Library(Default::default()),
            ScreenName::Details(id) => BrowserScreen::Details(DetailsState::new(id.to_string())),
            ScreenName::Search => BrowserScreen::Search(Default::default()),
            ScreenName::Artist(id) => BrowserScreen::Artist(ArtistState::new(id.to_string()))
        }
    }

    fn state(&mut self) -> &mut dyn UpdatableState<Action=BrowserAction, Event=BrowserEvent> {
        match self {
            Self::Library(state) => state,
            Self::Details(state) => state,
            Self::Search(state) => state,
            Self::Artist(state) => state
        }
    }
}

impl NamedScreen for BrowserScreen {

    type Name = ScreenName;

    fn name(&self) -> &Self::Name {
        match self {
            Self::Library(state) => &state.name,
            Self::Details(state) => &state.name,
            Self::Search(state) => &state.name,
            Self::Artist(state) => &state.name
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
    Current
}


struct NavStack<Screen>(Vec<Screen>);

impl <Screen> NavStack<Screen> where Screen: NamedScreen + Clone {

    fn new(initial: Screen) -> Self {
        Self(vec![initial])
    }

    fn iter_rev(&self) -> impl Iterator<Item=&Screen> {
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
        let split = self.0
            .iter()
            .position(|s| s.name() == name)
            .unwrap();
        self.0.truncate(split + 1);
    }

    fn screen_state(&self, name: &Screen::Name) -> ScreenState {
        self.0.iter()
            .rev()
            .enumerate()
            .find_map(|(i, screen)| {
                let is_screen = screen.name() == name;
                match (i, is_screen) {
                    (0, true) => Some(ScreenState::Current),
                    (_, true) => Some(ScreenState::Present),
                    (_, _) => None
                }
            })
            .unwrap_or(ScreenState::NotPresent)
    }
}

pub struct BrowserState {
    navigation: NavStack<BrowserScreen>
}

impl BrowserState {

    pub fn new() -> Self {
        Self { navigation: NavStack::new(BrowserScreen::Library(Default::default())) }
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

    pub fn library_state(&self) -> Option<&LibraryState> {
        self.navigation.iter_rev().find_map(|screen| {
            match screen {
                BrowserScreen::Library(state) => Some(state),
                _ => None
            }
        })
    }

    pub fn details_state(&self) -> Option<&DetailsState> {
        self.navigation.iter_rev().find_map(|screen| {
            match screen {
                BrowserScreen::Details(state) => Some(state),
                _ => None
            }
        })
    }

    pub fn search_state(&self) -> Option<&SearchState> {
        self.navigation.iter_rev().find_map(|screen| {
            match screen {
                BrowserScreen::Search(state) => Some(state),
                _ => None
            }
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
            },
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
            },
            BrowserAction::NavigationPush(name) => self.push_if_needed(name),
            BrowserAction::NavigationPop if can_pop => {
                self.navigation.pop();
                vec![BrowserEvent::NavigationPopped]
            },
            _ => self.navigation.current_mut().state().update_with(action)
        }
    }
}
