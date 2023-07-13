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
            Self::AlbumDetails(s) => Cow::Owned(format!("album_{s}")),
            Self::Search => Cow::Borrowed("search"),
            Self::Artist(s) => Cow::Owned(format!("artist_{s}")),
            Self::PlaylistDetails(s) => Cow::Owned(format!("playlist_{s}")),
            Self::User(s) => Cow::Owned(format!("user_{s}")),
        }
    }
}

impl PartialEq for ScreenName {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}

impl Eq for ScreenName {}

// ALBUM details
pub struct DetailsState {
    pub id: String,
    pub name: ScreenName,
    pub content: Option<AlbumFullDescription>,
    // Read the songs from here, not content (won't get more than the initial batch of songs)
    pub songs: SongListModel,
}

impl DetailsState {
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
            name: ScreenName::AlbumDetails(id),
            content: None,
            songs: SongListModel::new(50),
        }
    }
}

impl UpdatableState for DetailsState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.as_ref() {
            BrowserAction::SetAlbumDetails(album) if album.description.id == self.id => {
                let AlbumDescription { id, songs, .. } = album.description.clone();
                self.songs.add(songs).commit();
                self.content = Some(*album.clone());
                vec![BrowserEvent::AlbumDetailsLoaded(id)]
            }
            BrowserAction::AppendAlbumTracks(id, batch) if id == &self.id => {
                self.songs.add(*batch.clone()).commit();
                vec![BrowserEvent::AlbumTracksAppended(id.clone())]
            }
            BrowserAction::SaveAlbum(album) if album.id == self.id => {
                let id = album.id.clone();
                if let Some(album) = self.content.as_mut() {
                    album.description.is_liked = true;
                    vec![BrowserEvent::AlbumSaved(id)]
                } else {
                    vec![]
                }
            }
            BrowserAction::UnsaveAlbum(id) if id == &self.id => {
                if let Some(album) = self.content.as_mut() {
                    album.description.is_liked = false;
                    vec![BrowserEvent::AlbumUnsaved(id.clone())]
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
    // Read the songs from here, not content (won't get more than the initial batch of songs)
    pub songs: SongListModel,
}

impl PlaylistDetailsState {
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
            name: ScreenName::PlaylistDetails(id),
            playlist: None,
            songs: SongListModel::new(100),
        }
    }
}

impl UpdatableState for PlaylistDetailsState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.as_ref() {
            BrowserAction::SetPlaylistDetails(playlist) if playlist.id == self.id => {
                let PlaylistDescription { id, songs, .. } = *playlist.clone();
                self.songs.add(songs).commit();
                self.playlist = Some(*playlist.clone());
                vec![BrowserEvent::PlaylistDetailsLoaded(id)]
            }
            BrowserAction::UpdatePlaylistName(PlaylistSummary { id, title }) if id == &self.id => {
                if let Some(p) = self.playlist.as_mut() {
                    p.title = title.clone();
                }
                vec![BrowserEvent::PlaylistDetailsLoaded(self.id.clone())]
            }
            BrowserAction::AppendPlaylistTracks(id, song_batch) if id == &self.id => {
                self.songs.add(*song_batch.clone()).commit();
                vec![BrowserEvent::PlaylistTracksAppended(id.clone())]
            }
            BrowserAction::RemoveTracksFromPlaylist(id, uris) if id == &self.id => {
                self.songs.remove(&uris[..]).commit();
                vec![BrowserEvent::PlaylistTracksRemoved(self.id.clone())]
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
    pub top_tracks: SongListModel,
}

impl ArtistState {
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
            name: ScreenName::Artist(id.clone()),
            artist: None,
            next_page: Pagination::new(id, 20),
            albums: ListStore::new(),
            top_tracks: SongListModel::new(10),
        }
    }
}

impl UpdatableState for ArtistState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.as_ref() {
            BrowserAction::SetArtistDetails(details) if details.id == self.id => {
                let ArtistDescription {
                    id,
                    name,
                    albums,
                    mut top_tracks,
                } = *details.clone();
                self.artist = Some(name);
                self.albums
                    .replace_all(albums.into_iter().map(|a| a.into()));
                self.next_page.reset_count(self.albums.len());

                top_tracks.truncate(5);
                self.top_tracks.append(top_tracks).commit();

                vec![BrowserEvent::ArtistDetailsUpdated(id)]
            }
            BrowserAction::AppendArtistReleases(id, albums) if id == &self.id => {
                self.next_page.set_loaded_count(albums.len());
                self.albums.extend(albums.iter().map(|a| a.into()));
                vec![BrowserEvent::ArtistDetailsUpdated(self.id.clone())]
            }
            _ => vec![],
        }
    }
}

// The "home" represents screens visible initially (saved albums, saved playlists, saved tracks)
pub struct HomeState {
    pub name: ScreenName,
    pub visible_page: &'static str,
    pub next_albums_page: Pagination<()>,
    pub albums: ListStore<AlbumModel>,
    pub next_playlists_page: Pagination<()>,
    pub playlists: ListStore<AlbumModel>,
    pub saved_tracks: SongListModel,
    pub followed_artists: ListStore<ArtistModel>,
    pub next_followed_artists_page: Pagination<()>,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            name: ScreenName::Home,
            visible_page: "library",
            next_albums_page: Pagination::new((), 30),
            albums: ListStore::new(),
            next_playlists_page: Pagination::new((), 30),
            playlists: ListStore::new(),
            saved_tracks: SongListModel::new(50),
            followed_artists: ListStore::new(),
            next_followed_artists_page: Pagination::new((), 30)
        }
    }
}

impl UpdatableState for HomeState {
    type Action = BrowserAction;
    type Event = BrowserEvent;

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.as_ref() {
            BrowserAction::SetHomeVisiblePage(page) => {
                self.visible_page = *page;
                vec![BrowserEvent::HomeVisiblePageChanged(page)]
            }
            BrowserAction::SetLibraryContent(content) => {
                if !self.albums.eq(content, |a, b| a.uri() == b.id) {
                    self.albums.replace_all(content.iter().map(|a| a.into()));
                    self.next_albums_page.reset_count(self.albums.len());
                    vec![BrowserEvent::LibraryUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::PrependPlaylistsContent(content) => {
                self.playlists.prepend(content.iter().map(|a| a.into()));
                vec![BrowserEvent::SavedPlaylistsUpdated]
            }
            BrowserAction::AppendLibraryContent(content) => {
                self.next_albums_page.set_loaded_count(content.len());
                self.albums.extend(content.iter().map(|a| a.into()));
                vec![BrowserEvent::LibraryUpdated]
            }
            BrowserAction::SaveAlbum(album) => {
                let album_id = album.id.clone();
                let already_present = self.albums.iter().any(|a| a.uri() == album_id);
                if already_present {
                    vec![]
                } else {
                    self.albums.insert(0, (*album.clone()).into());
                    self.next_albums_page.increment();
                    vec![BrowserEvent::LibraryUpdated]
                }
            }
            BrowserAction::UnsaveAlbum(id) => {
                let position = self.albums.iter().position(|a| a.uri() == *id);
                if let Some(position) = position {
                    self.albums.remove(position as u32);
                    self.next_albums_page.decrement();
                    vec![BrowserEvent::LibraryUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::SetPlaylistsContent(content) => {
                if !self.playlists.eq(content, |a, b| a.uri() == b.id) {
                    self.playlists.replace_all(content.iter().map(|a| a.into()));
                    self.next_playlists_page.reset_count(self.playlists.len());
                    vec![BrowserEvent::SavedPlaylistsUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::AppendPlaylistsContent(content) => {
                self.next_playlists_page.set_loaded_count(content.len());
                self.playlists.extend(content.iter().map(|p| p.into()));
                vec![BrowserEvent::SavedPlaylistsUpdated]
            }
            BrowserAction::SetFollowedArtistsContent(content) => {
                if !self.followed_artists.eq(content, |a, b| a.id() == b.id) {
                    self.followed_artists.replace_all(content.iter().map(|a| a.into()));
                    self.next_followed_artists_page.reset_count(self.followed_artists.len());
                    vec![BrowserEvent::FollowedArtistsUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::AppendFollowedArtistsContent(content) => {
                self.next_followed_artists_page.set_loaded_count(content.len());
                self.followed_artists.extend(content.iter().map(|p| p.into()));
                vec![BrowserEvent::FollowedArtistsUpdated]
            }
            BrowserAction::UpdatePlaylistName(PlaylistSummary { id, title }) => {
                if let Some(p) = self.playlists.iter().find(|p| &p.uri() == id) {
                    p.set_album(title.to_owned());
                }
                vec![BrowserEvent::SavedPlaylistsUpdated]
            }
            BrowserAction::AppendSavedTracks(song_batch) => {
                if self.saved_tracks.add(*song_batch.clone()).commit() {
                    vec![BrowserEvent::SavedTracksUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::SetSavedTracks(song_batch) => {
                let song_batch = *song_batch.clone();
                if self
                    .saved_tracks
                    .clear()
                    .and(|s| s.add(song_batch))
                    .commit()
                {
                    vec![BrowserEvent::SavedTracksUpdated]
                } else {
                    vec![]
                }
            }
            BrowserAction::SaveTracks(tracks) => {
                self.saved_tracks.prepend(tracks.clone()).commit();
                vec![BrowserEvent::SavedTracksUpdated]
            }
            BrowserAction::RemoveSavedTracks(tracks) => {
                self.saved_tracks.remove(&tracks[..]).commit();
                vec![BrowserEvent::SavedTracksUpdated]
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

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.as_ref() {
            BrowserAction::Search(query) if query != &self.query => {
                self.query = query.clone();
                vec![BrowserEvent::SearchUpdated]
            }
            BrowserAction::SetSearchResults(results) => {
                self.album_results = results.albums.clone();
                self.artist_results = results.artists.clone();
                vec![BrowserEvent::SearchResultsUpdated]
            }
            _ => vec![],
        }
    }
}

// Screen when we click on the name of a playlist owner
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

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.as_ref() {
            BrowserAction::SetUserDetails(user) if user.id == self.id => {
                let UserDescription {
                    id,
                    name,
                    playlists,
                } = *user.clone();
                self.user = Some(name);
                self.playlists
                    .replace_all(playlists.iter().map(|p| p.into()));
                self.next_page.reset_count(self.playlists.len());

                vec![BrowserEvent::UserDetailsUpdated(id)]
            }
            BrowserAction::AppendUserPlaylists(id, playlists) if id == &self.id => {
                self.next_page.set_loaded_count(playlists.len());
                self.playlists.extend(playlists.iter().map(|p| p.into()));
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
        artist_state.update_with(Cow::Owned(BrowserAction::SetArtistDetails(Box::new(
            ArtistDescription {
                id: "id".to_owned(),
                name: "Foo".to_owned(),
                albums: vec![],
                top_tracks: vec![],
            },
        ))));

        let next = artist_state.next_page;
        assert_eq!(None, next.next_offset);
    }

    #[test]
    fn test_next_page_more() {
        let fake_album = AlbumDescription {
            id: "".to_owned(),
            title: "".to_owned(),
            artists: vec![],
            release_date: Some("1970-01-01".to_owned()),
            art: Some("".to_owned()),
            songs: SongBatch::empty(),
            is_liked: false,
        };
        let id = "id".to_string();
        let mut artist_state = ArtistState::new(id.clone());
        artist_state.update_with(Cow::Owned(BrowserAction::SetArtistDetails(Box::new(
            ArtistDescription {
                id: id.clone(),
                name: "Foo".to_owned(),
                albums: (0..20).map(|_| fake_album.clone()).collect(),
                top_tracks: vec![],
            },
        ))));

        let next = &artist_state.next_page;
        assert_eq!(Some(20), next.next_offset);

        artist_state.update_with(Cow::Owned(BrowserAction::AppendArtistReleases(
            id.clone(),
            vec![],
        )));

        let next = &artist_state.next_page;
        assert_eq!(None, next.next_offset);
    }
}
