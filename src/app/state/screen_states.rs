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
    artist_id: String,
    pub artist: Option<String>,
    pub page: u32,
    pub albums: ListStore<AlbumModel>,
    pub top_tracks: Vec<SongDescription>,
}

impl ArtistState {
    pub fn new(id: String) -> Self {
        Self {
            name: ScreenName::Artist(id.clone()),
            artist_id: id,
            artist: None,
            page: 0,
            albums: ListStore::new(),
            top_tracks: vec![],
        }
    }

    pub fn next_page(&self) -> Option<(String, u32, u32)> {
        let batch_size = 20;
        let offset = self.page * batch_size;
        let current_len = self.albums.len() as u32;
        if current_len < offset {
            None
        } else {
            Some((self.artist_id.clone(), offset, batch_size))
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

                self.page = 1;
                self.albums.remove_all();
                for album in albums {
                    self.albums.append(album.into());
                }

                top_tracks.truncate(5);
                self.top_tracks = top_tracks;

                vec![BrowserEvent::ArtistDetailsUpdated]
            }
            BrowserAction::AppendArtistReleases(albums) => {
                for album in albums {
                    self.albums.append(album.into());
                }
                self.page += 1;
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

        let next = artist_state.next_page();
        assert_eq!(false, next.is_some());
    }

    #[test]
    fn test_next_page_more() {
        let fake_album = AlbumDescription {
            id: "".to_owned(),
            title: "".to_owned(),
            artists: vec![],
            art: "".to_owned(),
            songs: vec![],
        };
        let mut artist_state = ArtistState::new("id".to_owned());
        artist_state.update_with(BrowserAction::SetArtistDetails(ArtistDescription {
            name: "Foo".to_owned(),
            albums: (0..20).map(|_| fake_album.clone()).collect(),
            top_tracks: vec![],
        }));

        let next = artist_state.next_page();
        assert_eq!(Some(("id".to_owned(), 20, 20)), next);
    }
}
