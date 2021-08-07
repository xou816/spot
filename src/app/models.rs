use std::convert::From;

pub use super::gtypes::*;
use crate::{api::client::AlbumInfo, app::components::utils::format_duration};

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
            &format_duration(self.duration.into()),
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
pub struct AlbumDetailedInfo {
    pub info: AlbumInfo,
    pub artists: String,
    pub art: Option<String>,
    pub total_time: u32,
    pub is_liked: bool,
}

impl AlbumDetailedInfo {
    pub fn new(info: AlbumInfo, detail: AlbumDescription) -> Self {
        let artists = detail.artists_name();
        let total_time = detail.songs.iter().map(|a| a.duration).sum();
        Self {
            info,
            artists,
            art: detail.art.to_owned(),
            total_time,
            is_liked: detail.is_liked,
        }
    }
    pub fn formatted_time(&self) -> String {
        let duration = std::time::Duration::from_millis(self.total_time.into());
        let millis = duration.as_millis() % 1000;
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        let hours = (duration.as_secs() / 60) / 60;
        if hours > 0 {
            format!("{}:{:02}:{:02} + {}ms", hours, minutes, seconds, millis)
        } else {
            format!("{}:{:02} + {}ms", minutes, seconds, millis)
        }
    }

    pub fn markets(&self) -> String {
        self.info
            .available_markets
            .iter()
            .map(|m| m.0.to_owned())
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn copyrights(&self) -> String {
        self.info
            .copyrights
            .iter()
            .map(|c| format!("[{}] {}", c.type_, c.text))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

#[derive(Clone, Debug)]
pub struct PlaylistDescription {
    pub id: String,
    pub title: String,
    pub art: Option<String>,
    pub songs: Vec<SongDescription>,
    pub last_batch: Batch,
    pub owner: UserRef,
}

#[derive(Clone, Debug)]
pub struct PlaylistSummary {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct SongDescription {
    pub id: String,
    pub uri: String,
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

#[derive(Clone, Copy, Debug)]
pub struct Batch {
    pub offset: usize,
    pub batch_size: usize,
    pub total: usize,
}

impl Batch {
    pub fn next(self) -> Option<Self> {
        let Self {
            offset,
            batch_size,
            total,
        } = self;

        Some(Self {
            offset: offset + batch_size,
            batch_size,
            total,
        })
        .filter(|b| b.offset < total)
    }
}

#[derive(Debug, Clone)]
pub struct SongBatch {
    pub songs: Vec<SongDescription>,
    pub batch: Batch,
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

#[derive(Clone, Debug)]
pub struct UserDescription {
    pub id: String,
    pub name: String,
    pub playlists: Vec<PlaylistDescription>,
}
