use std::borrow::Cow;
use std::cmp::PartialEq;

use super::{BrowserAction, BrowserEvent, UpdatableState};
use crate::app::models::*;
use crate::app::ListStore;

#[derive(Clone, Debug)]
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

impl PartialEq for ScreenName {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}

impl Eq for ScreenName {}

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
            BrowserAction::SaveAlbum(album) => {
                let id = album.id;
                if let Some(mut album) = self.content.as_mut() {
                    album.is_liked = true;
                    vec![BrowserEvent::AlbumSaved(id)]
                } else {
                    vec![]
                }
            }
            BrowserAction::UnsaveAlbum(id) => {
                if let Some(mut album) = self.content.as_mut() {
                    album.is_liked = false;
                    vec![BrowserEvent::AlbumUnsaved(id)]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}

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

pub struct Pagination<T> {
    pub data: T,
    pub next_offset: Option<u32>,
    pub batch_size: u32,
}

impl<T> Pagination<T> {
    fn new(data: T, batch_size: u32) -> Self {
        Self {
            data,
            next_offset: Some(0),
            batch_size,
        }
    }

    fn reset(&mut self, new_length: u32) {
        self.next_offset = if new_length >= self.batch_size {
            Some(self.batch_size)
        } else {
            None
        }
    }

    fn update(&mut self, new_length: u32) {
        if let Some(offset) = self.next_offset.take() {
            self.next_offset = if new_length >= offset + self.batch_size {
                Some(offset + self.batch_size)
            } else {
                None
            }
        }
    }

    fn decrement(&mut self) {
        if let Some(offset) = self.next_offset.take() {
            self.next_offset = Some(offset - 1);
        }
    }

    fn increment(&mut self) {
        if let Some(offset) = self.next_offset.take() {
            self.next_offset = Some(offset + 1);
        }
    }
}

pub struct ArtistState {
    pub name: ScreenName,
    pub artist: Option<String>,
    pub next_page: Pagination<String>,
    pub albums: ListStore<AlbumModel>,
    pub top_tracks: Vec<SongDescription>,
}

impl ArtistState {
    pub fn new(id: String) -> Self {
        Self {
            name: ScreenName::Artist(id.clone()),
            artist: None,
            next_page: Pagination::new(id, 20),
            albums: ListStore::new(),
            top_tracks: vec![],
        }
    }
}

impl UpdatableState for ArtistState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetArtistDetails(ArtistDescription {
                name,
                albums,
                mut top_tracks,
            }) => {
                self.artist = Some(name);

                self.albums.remove_all();
                for album in albums {
                    self.albums.append(album.into());
                }
                self.next_page.reset(self.albums.len() as u32);

                top_tracks.truncate(5);
                self.top_tracks = top_tracks;

                vec![BrowserEvent::ArtistDetailsUpdated]
            }
            BrowserAction::AppendArtistReleases(albums) => {
                for album in albums {
                    self.albums.append(album.into());
                }
                self.next_page.update(self.albums.len() as u32);
                vec![BrowserEvent::ArtistDetailsUpdated]
            }
            _ => vec![],
        }
    }
}

pub struct HomeState {
    pub name: ScreenName,
    pub next_albums_page: Pagination<()>,
    pub albums: ListStore<AlbumModel>,
    pub next_playlists_page: Pagination<()>,
    pub playlists: ListStore<AlbumModel>,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            name: ScreenName::Home,
            next_albums_page: Pagination::new((), 30),
            albums: ListStore::new(),
            next_playlists_page: Pagination::new((), 30),
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
                    self.albums.remove_all();
                    for album in converted {
                        self.albums.append(album);
                    }
                    self.next_albums_page.reset(self.albums.len() as u32);
                    vec![BrowserEvent::LibraryUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::AppendLibraryContent(content) => {
                for album in content {
                    self.albums.append(album.into());
                }
                self.next_albums_page.update(self.albums.len() as u32);
                vec![BrowserEvent::LibraryUpdated]
            }
            BrowserAction::SaveAlbum(album) => {
                let album_id = album.id.clone();
                let already_present = self
                    .albums
                    .iter()
                    .find(|a| a.uri().as_ref() == Some(&album_id))
                    .is_some();
                if already_present {
                    vec![]
                } else {
                    self.albums.insert(0, album.into());
                    self.next_albums_page.increment();
                    vec![BrowserEvent::LibraryUpdated]
                }
            }
            BrowserAction::UnsaveAlbum(id) => {
                let position = self
                    .albums
                    .iter()
                    .position(|a| a.uri().as_ref() == Some(&id));
                if let Some(position) = position {
                    self.albums.remove(position as u32);
                    self.next_albums_page.decrement();
                    vec![BrowserEvent::LibraryUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::SetPlaylistsContent(content) => {
                let converted = content
                    .iter()
                    .map(|a| a.into())
                    .collect::<Vec<AlbumModel>>();
                if !self.playlists.eq(&converted, |a, b| a.uri() == b.uri()) {
                    self.playlists.remove_all();
                    for playlist in converted {
                        self.playlists.append(playlist);
                    }
                    self.next_playlists_page.reset(self.playlists.len() as u32);
                    vec![BrowserEvent::SavedPlaylistsUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::AppendPlaylistsContent(content) => {
                for playlist in content {
                    self.playlists.append(playlist.into());
                }
                self.next_playlists_page.update(self.playlists.len() as u32);
                vec![BrowserEvent::SavedPlaylistsUpdated]
            }
            _ => vec![],
        }
    }
}

pub struct SearchState {
    pub name: ScreenName,
    pub query: String,
    pub album_results: Vec<AlbumDescription>,
    pub artist_results: Vec<ArtistSummary>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            name: ScreenName::Search,
            query: "".to_owned(),
            album_results: vec![],
            artist_results: vec![],
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
                self.album_results = results.albums;
                self.artist_results = results.artists;
                vec![BrowserEvent::SearchResultsUpdated]
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_next_page_no_next() {
        let mut artist_state = ArtistState::new("id".to_owned());
        artist_state.update_with(BrowserAction::SetArtistDetails(ArtistDescription {
            name: "Foo".to_owned(),
            albums: vec![],
            top_tracks: vec![],
        }));

        let next = artist_state.next_page;
        assert_eq!(None, next.next_offset);
    }

    #[test]
    fn test_next_page_more() {
        let fake_album = AlbumDescription {
            id: "".to_owned(),
            title: "".to_owned(),
            artists: vec![],
            art: "".to_owned(),
            songs: vec![],
            is_liked: false,
        };
        let mut artist_state = ArtistState::new("id".to_owned());
        artist_state.update_with(BrowserAction::SetArtistDetails(ArtistDescription {
            name: "Foo".to_owned(),
            albums: (0..20).map(|_| fake_album.clone()).collect(),
            top_tracks: vec![],
        }));

        let next = &artist_state.next_page;
        assert_eq!(Some(20), next.next_offset);

        artist_state.update_with(BrowserAction::AppendArtistReleases(vec![]));

        let next = &artist_state.next_page;
        assert_eq!(None, next.next_offset);
    }
}
