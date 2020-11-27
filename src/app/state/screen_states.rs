use crate::app::models::*;
use super::{BrowserEvent, BrowserAction, UpdatableState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScreenName {
    Library, Details(String), Search
}

impl ScreenName {
    pub fn identifier(&self) -> &str {
        match self {
            Self::Library => "library",
            Self::Details(s) => &s[..],
            Self::Search => "search"
        }
    }
}

#[derive(Clone)]
pub struct DetailsState {
    pub name: ScreenName,
    pub content: Option<AlbumDescription>
}

impl DetailsState {
    pub fn new(id: String) -> Self {
        Self { name: ScreenName::Details(id), content: None }
    }
}

impl UpdatableState for DetailsState {

    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetDetails(album) => {
                self.content = Some(album);
                vec![BrowserEvent::DetailsLoaded]
            },
            _ => vec![]
        }
    }
}

#[derive(Clone)]
pub struct LibraryState {
    pub name: ScreenName,
    pub page: u32,
    pub albums: Vec<AlbumDescription>
}

impl Default for LibraryState {
    fn default() -> Self {
        Self { name: ScreenName::Library, page: 0, albums: vec![] }
    }
}

impl UpdatableState for LibraryState {

    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetContent(content) if content != self.albums => {
                self.page = 1;
                self.albums = content;
                vec![BrowserEvent::ContentSet]
            },
            BrowserAction::AppendContent(mut content) => {
                self.page += 1;
                let append_index = self.albums.len();
                self.albums.append(content.as_mut());
                vec![BrowserEvent::ContentAppended(append_index)]
            },
            _ => vec![]
        }
    }
}

#[derive(Clone)]
pub struct SearchState {
    pub name: ScreenName,
    pub query: String,
    pub album_results: Vec<AlbumDescription>
}

impl Default for SearchState {
    fn default() -> Self {
        Self { name: ScreenName::Search, query: "".to_owned(), album_results: vec![] }
    }
}


impl UpdatableState for SearchState {

    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::Search(query) if query != self.query => {
                self.query = query;
                vec![BrowserEvent::SearchUpdated]
            },
            BrowserAction::SetSearchResults(results) => {
                self.album_results = results;
                vec![BrowserEvent::SearchResultsUpdated]
            }
            _ => vec![]
        }
    }
}
