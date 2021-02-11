use super::{BrowserAction, BrowserEvent, UpdatableState};
use crate::app::models::*;
use crate::app::ListStore;
use std::borrow::Cow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScreenName {
    Home,
    AlbumDetails(String),
    Search,
    Artist(String),
    PlaylistDetails(String),
}

impl ScreenName {
    pub fn identifier(&self) -> Cow<str> {
        match self {
            Self::Home => Cow::Borrowed("home"),
            Self::AlbumDetails(s) => Cow::Owned(format!("album_{}", s)),
            Self::Search => Cow::Borrowed("search"),
            Self::Artist(s) => Cow::Owned(format!("artist_{}", s)),
            Self::PlaylistDetails(s) => Cow::Owned(format!("playlist_{}", s)),
        }
    }
}

#[derive(Clone)]
pub struct DetailsState {
    pub name: ScreenName,
    pub content: Option<AlbumDescription>,
}

impl DetailsState {
    pub fn new(id: String) -> Self {
        Self {
            name: ScreenName::AlbumDetails(id),
            content: None,
        }
    }
}

impl UpdatableState for DetailsState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetAlbumDetails(album) => {
                self.content = Some(album);
                vec![BrowserEvent::AlbumDetailsLoaded]
            }
            _ => vec![],
        }
    }
}

#[derive(Clone)]
pub struct PlaylistDetailsState {
    pub name: ScreenName,
    pub content: Option<PlaylistDescription>,
}

impl PlaylistDetailsState {
    pub fn new(id: String) -> Self {
        Self {
            name: ScreenName::PlaylistDetails(id),
            content: None,
        }
    }
}

impl UpdatableState for PlaylistDetailsState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetPlaylistDetails(playlist) => {
                self.content = Some(playlist);
                vec![BrowserEvent::PlaylistDetailsLoaded]
            }
            _ => vec![],
        }
    }
}

#[derive(Clone)]
pub struct ArtistState {
    pub name: ScreenName,
    pub artist: Option<String>,
    pub albums: ListStore<AlbumModel>,
}

impl ArtistState {
    pub fn new(id: String) -> Self {
        Self {
            name: ScreenName::Artist(id),
            artist: None,
            albums: ListStore::new(),
        }
    }
}

impl UpdatableState for ArtistState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetArtistDetails(details) => {
                self.artist = Some(details.name);
                for album in details.albums {
                    self.albums.append(album.into());
                }
                vec![BrowserEvent::ArtistDetailsUpdated]
            }
            _ => vec![],
        }
    }
}

#[derive(Clone)]
pub struct HomeState {
    pub name: ScreenName,
    pub albums_page: u32,
    pub albums: ListStore<AlbumModel>,
    pub playlists_page: u32,
    pub playlists: ListStore<AlbumModel>,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            name: ScreenName::Home,
            albums_page: 0,
            albums: ListStore::new(),
            playlists_page: 0,
            playlists: ListStore::new(),
        }
    }
}

impl UpdatableState for HomeState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetLibraryContent(content) => {
                let converted = content
                    .iter()
                    .map(|a| a.into())
                    .collect::<Vec<AlbumModel>>();
                if !self.albums.eq(&converted, |a, b| a.uri() == b.uri()) {
                    self.albums_page = 1;
                    self.albums.remove_all();
                    for album in converted {
                        self.albums.append(album);
                    }
                    vec![BrowserEvent::LibraryUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::AppendLibraryContent(content) => {
                self.albums_page += 1;
                for album in content {
                    self.albums.append(album.into());
                }
                vec![BrowserEvent::LibraryUpdated]
            }
            BrowserAction::SetPlaylistsContent(content) => {
                let converted = content
                    .iter()
                    .map(|a| a.into())
                    .collect::<Vec<AlbumModel>>();
                if !self.playlists.eq(&converted, |a, b| a.uri() == b.uri()) {
                    self.playlists_page = 1;
                    self.playlists.remove_all();
                    for playlist in converted {
                        self.playlists.append(playlist);
                    }
                    vec![BrowserEvent::SavedPlaylistsUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::AppendPlaylistsContent(content) => {
                self.playlists_page += 1;
                for playlist in content {
                    self.playlists.append(playlist.into());
                }
                vec![BrowserEvent::SavedPlaylistsUpdated]
            }
            _ => vec![],
        }
    }
}

#[derive(Clone)]
pub struct SearchState {
    pub name: ScreenName,
    pub query: String,
    pub album_results: Vec<AlbumDescription>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            name: ScreenName::Search,
            query: "".to_owned(),
            album_results: vec![],
        }
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
            }
            BrowserAction::SetSearchResults(results) => {
                self.album_results = results;
                vec![BrowserEvent::SearchResultsUpdated]
            }
            _ => vec![],
        }
    }
}
