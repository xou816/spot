pub use super::gtypes::*;
use std::convert::From;

impl From<&AlbumDescription> for AlbumModel {
    fn from(album: &AlbumDescription) -> Self {
        AlbumModel::new(&album.artists_name(), &album.title, &album.art, &album.id)
    }
}

impl From<AlbumDescription> for AlbumModel {
    fn from(album: AlbumDescription) -> Self {
        Self::from(&album)
    }
}

impl From<&PlaylistDescription> for AlbumModel {
    fn from(playlist: &PlaylistDescription) -> Self {
        AlbumModel::new(
            &playlist.owner.display_name,
            &playlist.title,
            &playlist.art,
            &playlist.id,
        )
    }
}

impl From<PlaylistDescription> for AlbumModel {
    fn from(playlist: PlaylistDescription) -> Self {
        Self::from(&playlist)
    }
}

impl SongDescription {
    pub fn to_song_model(&self, position: usize) -> SongModel {
        SongModel::new(
            &self.id,
            (position + 1) as u32,
            &self.title,
            &self.artists_name(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct UserRef {
    pub id: String,
    pub display_name: String,
}

#[derive(Clone, Debug)]
pub struct ArtistRef {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct AlbumRef {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct SearchResults {
    pub albums: Vec<AlbumDescription>,
    pub artists: Vec<ArtistSummary>,
}

#[derive(Clone, Debug)]
pub struct AlbumDescription {
    pub id: String,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub art: Option<String>,
    pub songs: Vec<SongDescription>,
    pub is_liked: bool,
}

impl AlbumDescription {
    pub fn artists_name(&self) -> String {
        self.artists
            .iter()
            .map(|a| a.name.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

impl PartialEq for AlbumDescription {
    fn eq(&self, other: &AlbumDescription) -> bool {
        self.id == other.id
    }
}

impl Eq for AlbumDescription {}

#[derive(Clone, Debug)]
pub struct PlaylistDescription {
    pub id: String,
    pub title: String,
    pub art: Option<String>,
    pub songs: Vec<SongDescription>,
    pub owner: UserRef,
}

#[derive(Clone, Debug)]
pub struct SongDescription {
    pub id: String,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub album: AlbumRef,
    pub duration: u32,
    pub art: Option<String>,
}

impl SongDescription {
    pub fn artists_name(&self) -> String {
        self.artists
            .iter()
            .map(|a| a.name.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

#[derive(Clone, Debug)]
pub struct ArtistDescription {
    pub id: String,
    pub name: String,
    pub albums: Vec<AlbumDescription>,
    pub top_tracks: Vec<SongDescription>,
}

#[derive(Clone, Debug)]
pub struct ArtistSummary {
    pub id: String,
    pub name: String,
    pub photo: Option<String>,
}
