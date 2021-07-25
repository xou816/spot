use std::{collections::HashMap, convert::From};

pub use super::gtypes::*;
use super::{BatchQuery, SongsSource};
use crate::app::components::utils::format_duration;

struct SongListBatch {
    batch: Batch,
    ids: Vec<String>,
}

struct SongList {
    batches: HashMap<usize, SongListBatch>,
    indexed_songs: HashMap<String, SongDescription>,
}

impl SongList {
    fn iter(&self) -> impl Iterator<Item = &'_ SongDescription> {
        let indexed_songs = &self.indexed_songs;
        self.batches.iter().flat_map(move |(batch_id, song)| {
            song.ids.iter().filter_map(move |id| indexed_songs.get(id))
        })
    }

    fn get(&self, i: usize) {}
}

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
    pub last_batch: Batch,
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
    pub fn formatted_time(&self) -> String {
        let duration: u32 = self.songs.iter().map(|song| song.duration).sum();
        format_duration(duration.into())
    }
}

#[derive(Clone, Debug)]
pub struct AlbumFullDescription {
    pub description: AlbumDescription,
    pub release_details: AlbumReleaseDetails,
}

#[derive(Clone, Debug)]
pub struct AlbumReleaseDetails {
    pub label: String,
    pub release_date: String,
    pub copyrights: Vec<CopyrightDetails>,
}

impl AlbumReleaseDetails {
    pub fn copyrights(&self) -> String {
        self.copyrights
            .iter()
            .map(|c| format!("[{}] {}", c.type_, c.text))
            .collect::<Vec<String>>()
            .join(",\n ")
    }
}

#[derive(Clone, Debug)]
pub struct CopyrightDetails {
    pub text: String,
    pub type_: char,
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
    pub fn first_of_size(batch_size: usize) -> Self {
        Self {
            offset: 0,
            batch_size,
            total: usize::MAX,
        }
    }

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

impl SongBatch {
    pub fn empty() -> Self {
        Self {
            songs: vec![],
            batch: Batch::first_of_size(0),
        }
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

#[derive(Clone, Debug)]
pub struct UserDescription {
    pub id: String,
    pub name: String,
    pub playlists: Vec<PlaylistDescription>,
}

impl From<&AlbumDescription> for BatchQuery {
    fn from(album: &AlbumDescription) -> Self {
        BatchQuery {
            source: SongsSource::Album(album.id.clone()),
            batch: album.last_batch,
        }
    }
}

impl From<&PlaylistDescription> for BatchQuery {
    fn from(playlist: &PlaylistDescription) -> Self {
        BatchQuery {
            source: SongsSource::Playlist(playlist.id.clone()),
            batch: playlist.last_batch,
        }
    }
}

impl From<&AlbumDescription> for SongBatch {
    fn from(album: &AlbumDescription) -> Self {
        SongBatch {
            songs: album.songs.clone(),
            batch: album.last_batch,
        }
    }
}

impl From<&PlaylistDescription> for SongBatch {
    fn from(playlist: &PlaylistDescription) -> Self {
        SongBatch {
            songs: playlist.songs.clone(),
            batch: playlist.last_batch,
        }
    }
}
