use std::convert::From;

use super::core::{Batch, SongList};
use super::gtypes::*;

use crate::app::components::utils::format_duration;

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
    pub songs: SongList,
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
    pub songs: SongList,
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

    pub fn resize(self, batch_size: usize) -> Vec<Self> {
        let SongBatch { mut songs, batch } = self;
        if batch_size > batch.batch_size {
            let new_batch = Batch {
                batch_size,
                ..batch
            };
            vec![Self {
                songs,
                batch: new_batch,
            }]
        } else {
            let n = songs.len();
            let iter_count = n / batch_size + (if n % batch_size > 0 { 1 } else { 0 });
            (0..iter_count)
                .map(|i| {
                    let offset = batch.offset + i * batch_size;
                    let new_batch = Batch {
                        offset,
                        total: batch.total,
                        batch_size,
                    };
                    let drain_upper = usize::min(batch_size, songs.len());
                    let new_songs = songs.drain(0..drain_upper).collect();
                    Self {
                        songs: new_songs,
                        batch: new_batch,
                    }
                })
                .collect()
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
