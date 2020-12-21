use std::borrow::Cow;
use crate::app::models::*;
use crate::app::ListStore;
use super::{BrowserEvent, BrowserAction, UpdatableState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScreenName {
    Library, Details(String), Search, Artist(String)
}

impl ScreenName {
    pub fn identifier<'a>(&'a self) -> Cow<'a, str> {
        match self {
            Self::Library => Cow::Borrowed("library"),
            Self::Details(s) => Cow::Owned(format!("album_{}", s)),
            Self::Search => Cow::Borrowed("search"),
            Self::Artist(s) => Cow::Owned(format!("artist_{}", s))
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
pub struct ArtistState {
    pub name: ScreenName,
    pub content: Option<ArtistDescription>
}

impl ArtistState {
    pub fn new(id: String) -> Self {
        Self { name: ScreenName::Artist(id), content: None }
    }
}

impl UpdatableState for ArtistState {

    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            _ => vec![]
        }
    }
}

#[derive(Clone)]
pub struct LibraryState {
    pub name: ScreenName,
    pub page: u32,
    pub albums: ListStore<AlbumModel>
}

impl Default for LibraryState {
    fn default() -> Self {
        Self { name: ScreenName::Library, page: 0, albums: ListStore::new() }
    }
}

impl UpdatableState for LibraryState {

    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetContent(content) => {
                let converted = content.iter().map(|a| a.into()).collect::<Vec<AlbumModel>>();
                if !self.albums.eq(&converted, |a, b| a.uri() == b.uri()) {
                    self.page = 1;
                    self.albums.remove_all();
                    for album in converted {
                        self.albums.append(album);
                    }
                    vec![BrowserEvent::LibraryUpdated]
                } else {
                    vec![]
                }
            },
            BrowserAction::AppendContent(content) => {
                self.page += 1;
                for album in content {
                    self.albums.append(album.into());
                }
                vec![BrowserEvent::LibraryUpdated]
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
