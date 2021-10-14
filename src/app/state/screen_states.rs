use std::borrow::Cow;
use std::cmp::PartialEq;

use super::{pagination::Pagination, BrowserAction, BrowserEvent, UpdatableState};
use crate::app::models::*;
use crate::app::ListStore;

#[derive(Clone, Debug)]
pub enum ScreenName {
    Home,
    AlbumDetails(String),
    Search,
    Artist(String),
    PlaylistDetails(String),
    User(String),
}

impl ScreenName {
    pub fn identifier(&self) -> Cow<str> {
        match self {
            Self::Home => Cow::Borrowed("home"),
            Self::AlbumDetails(s) => Cow::Owned(format!("album_{}", s)),
            Self::Search => Cow::Borrowed("search"),
            Self::Artist(s) => Cow::Owned(format!("artist_{}", s)),
            Self::PlaylistDetails(s) => Cow::Owned(format!("playlist_{}", s)),
            Self::User(s) => Cow::Owned(format!("user_{}", s)),
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
    pub id: String,
    pub name: ScreenName,
    pub content: Option<AlbumFullDescription>,
}

impl DetailsState {
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
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
            BrowserAction::SetAlbumDetails(album) if album.description.id == self.id => {
                let id = album.description.id.clone();
                self.content = Some(*album);
                vec![BrowserEvent::AlbumDetailsLoaded(id)]
            }
            BrowserAction::AppendAlbumTracks(id, batch) if id == self.id => {
                let offset = batch.batch.offset;
                if self
                    .content
                    .as_mut()
                    .and_then(|content| content.description.songs.add(*batch))
                    .is_some()
                {
                    vec![BrowserEvent::AlbumTracksAppended(id, offset)]
                } else {
                    vec![]
                }
            }
            BrowserAction::SaveAlbum(album) if album.id == self.id => {
                let id = album.id;
                if let Some(mut album) = self.content.as_mut() {
                    album.description.is_liked = true;
                    vec![BrowserEvent::AlbumSaved(id)]
                } else {
                    vec![]
                }
            }
            BrowserAction::UnsaveAlbum(id) if id == self.id => {
                if let Some(mut album) = self.content.as_mut() {
                    album.description.is_liked = false;
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
    pub id: String,
    pub name: ScreenName,
    pub playlist: Option<PlaylistDescription>,
}

impl PlaylistDetailsState {
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
            name: ScreenName::PlaylistDetails(id),
            playlist: None,
        }
    }
}

impl UpdatableState for PlaylistDetailsState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetPlaylistDetails(playlist) => {
                let id = playlist.id.clone();
                self.playlist = Some(*playlist);
                vec![BrowserEvent::PlaylistDetailsLoaded(id)]
            }
            BrowserAction::AppendPlaylistTracks(id, song_batch) if id == self.id => {
                let offset = song_batch.batch.offset;
                if self
                    .playlist
                    .as_mut()
                    .and_then(|playlist| playlist.songs.add(*song_batch))
                    .is_some()
                {
                    vec![BrowserEvent::PlaylistTracksAppended(id, offset)]
                } else {
                    vec![]
                }
            }
            BrowserAction::RemoveTracksFromPlaylist(uris) => {
                if let Some(playlist) = self.playlist.as_mut() {
                    let id = playlist.id.clone();
                    playlist.songs.remove(&uris[..]);
                    vec![BrowserEvent::PlaylistTracksRemoved(id, uris)]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}

pub struct ArtistState {
    pub id: String,
    pub name: ScreenName,
    pub artist: Option<String>,
    pub next_page: Pagination<String>,
    pub albums: ListStore<AlbumModel>,
    pub top_tracks: Vec<SongDescription>,
}

impl ArtistState {
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
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
            BrowserAction::SetArtistDetails(details) => {
                let ArtistDescription {
                    id,
                    name,
                    albums,
                    mut top_tracks,
                } = *details;
                self.artist = Some(name);
                self.albums
                    .replace_all(albums.into_iter().map(|a| a.into()));
                self.next_page.reset_count(self.albums.len());

                top_tracks.truncate(5);
                self.top_tracks = top_tracks;

                vec![BrowserEvent::ArtistDetailsUpdated(id)]
            }
            BrowserAction::AppendArtistReleases(albums) => {
                self.next_page.set_loaded_count(albums.len());
                self.albums.extend(albums.into_iter().map(|a| a.into()));
                vec![BrowserEvent::ArtistDetailsUpdated(self.id.clone())]
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
    pub saved_tracks: SongList,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            name: ScreenName::Home,
            next_albums_page: Pagination::new((), 30),
            albums: ListStore::new(),
            next_playlists_page: Pagination::new((), 30),
            playlists: ListStore::new(),
            saved_tracks: SongList::new_sized(50),
        }
    }
}

impl UpdatableState for HomeState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetLibraryContent(content) => {
                if !self
                    .albums
                    .eq(&content, |a, b| a.uri().as_ref() == Some(&b.id))
                {
                    self.albums
                        .replace_all(content.into_iter().map(|a| a.into()));
                    self.next_albums_page.reset_count(self.albums.len());
                    vec![BrowserEvent::LibraryUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::AppendLibraryContent(content) => {
                self.next_albums_page.set_loaded_count(content.len());
                self.albums.extend(content.into_iter().map(|a| a.into()));
                vec![BrowserEvent::LibraryUpdated]
            }
            BrowserAction::SaveAlbum(album) => {
                let album_id = album.id.clone();
                let already_present = self
                    .albums
                    .iter()
                    .any(|a| a.uri().as_ref() == Some(&album_id));
                if already_present {
                    vec![]
                } else {
                    self.albums.insert(0, (*album).into());
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
                if !self
                    .playlists
                    .eq(&content, |a, b| a.uri().as_ref() == Some(&b.id))
                {
                    self.playlists
                        .replace_all(content.into_iter().map(|a| a.into()));
                    self.next_playlists_page.reset_count(self.playlists.len());
                    vec![BrowserEvent::SavedPlaylistsUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::AppendPlaylistsContent(content) => {
                self.next_playlists_page.set_loaded_count(content.len());
                self.playlists.extend(content.into_iter().map(|p| p.into()));
                vec![BrowserEvent::SavedPlaylistsUpdated]
            }
            BrowserAction::AppendSavedTracks(song_batch) => {
                let offset = song_batch.batch.offset;
                if self.saved_tracks.add(*song_batch).is_some() {
                    vec![BrowserEvent::SavedTracksAppended(offset)]
                } else {
                    vec![]
                }
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

pub struct UserState {
    pub id: String,
    pub name: ScreenName,
    pub user: Option<String>,
    pub next_page: Pagination<String>,
    pub playlists: ListStore<AlbumModel>,
}

impl UserState {
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
            name: ScreenName::User(id.clone()),
            user: None,
            next_page: Pagination::new(id, 30),
            playlists: ListStore::new(),
        }
    }
}

impl UpdatableState for UserState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event> {
        match action {
            BrowserAction::SetUserDetails(user) => {
                let UserDescription {
                    id,
                    name,
                    playlists,
                } = *user;
                self.user = Some(name);
                self.playlists
                    .replace_all(playlists.into_iter().map(|p| p.into()));
                self.next_page.reset_count(self.playlists.len());

                vec![BrowserEvent::UserDetailsUpdated(id)]
            }
            BrowserAction::AppendUserPlaylists(playlists) => {
                self.next_page.set_loaded_count(playlists.len());
                self.playlists
                    .extend(playlists.into_iter().map(|p| p.into()));
                vec![BrowserEvent::UserDetailsUpdated(self.id.clone())]
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
        artist_state.update_with(BrowserAction::SetArtistDetails(Box::new(
            ArtistDescription {
                id: "".to_owned(),
                name: "Foo".to_owned(),
                albums: vec![],
                top_tracks: vec![],
            },
        )));

        let next = artist_state.next_page;
        assert_eq!(None, next.next_offset);
    }

    #[test]
    fn test_next_page_more() {
        let fake_album = AlbumDescription {
            id: "".to_owned(),
            title: "".to_owned(),
            artists: vec![],
            art: Some("".to_owned()),
            songs: SongList::new_sized(100),
            is_liked: false,
        };
        let mut artist_state = ArtistState::new("id".to_owned());
        artist_state.update_with(BrowserAction::SetArtistDetails(Box::new(
            ArtistDescription {
                id: "".to_owned(),
                name: "Foo".to_owned(),
                albums: (0..20).map(|_| fake_album.clone()).collect(),
                top_tracks: vec![],
            },
        )));

        let next = &artist_state.next_page;
        assert_eq!(Some(20), next.next_offset);

        artist_state.update_with(BrowserAction::AppendArtistReleases(vec![]));

        let next = &artist_state.next_page;
        assert_eq!(None, next.next_offset);
    }
}
