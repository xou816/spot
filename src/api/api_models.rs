use form_urlencoded::Serializer;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    convert::{Into, TryFrom, TryInto},
    vec::IntoIter,
};

use crate::app::models::*;

#[derive(Serialize)]
pub struct Uris {
    pub uris: Vec<String>,
}

pub enum SearchType {
    Artist,
    Album,
}

impl SearchType {
    fn into_string(self) -> &'static str {
        match self {
            Self::Artist => "artist",
            Self::Album => "album",
        }
    }
}

pub struct SearchQuery {
    pub query: String,
    pub types: Vec<SearchType>,
    pub limit: usize,
    pub offset: usize,
}

impl SearchQuery {
    pub fn into_query_string(self) -> String {
        let mut types = self
            .types
            .into_iter()
            .fold(String::new(), |acc, t| acc + t.into_string() + ",");
        types.pop();

        let re = Regex::new(r"(\W|\s)+").unwrap();
        let query = re.replace_all(&self.query[..], " ");

        let serialized = Serializer::new(String::new())
            .append_pair("q", query.as_ref())
            .append_pair("offset", &self.offset.to_string()[..])
            .append_pair("limit", &self.limit.to_string()[..])
            .append_pair("market", "from_token")
            .finish();

        format!("type={}&{}", types, serialized)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Page<T> {
    items: Option<Vec<T>>,
    offset: Option<usize>,
    limit: Option<usize>,
    total: usize,
}

impl<T> Page<T> {
    fn new(items: Vec<T>) -> Self {
        let l = items.len();
        Self {
            total: l,
            items: Some(items),
            offset: Some(0),
            limit: Some(l),
        }
    }

    pub fn limit(&self) -> usize {
        self.limit
            .or_else(|| Some(self.items.as_ref()?.len()))
            .unwrap_or(50)
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn offset(&self) -> usize {
        self.offset.unwrap_or(0)
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.unwrap_or_else(Vec::new).into_iter()
    }
}

impl<T> Default for Page<T> {
    fn default() -> Self {
        Self {
            items: None,
            total: 0,
            offset: Some(0),
            limit: Some(0),
        }
    }
}

trait WithImages {
    fn images(&self) -> &[Image];

    fn best_image<T: PartialOrd, F: Fn(&Image) -> T>(&self, criterion: F) -> Option<&Image> {
        let mut ords = self
            .images()
            .iter()
            .map(|image| (criterion(image), image))
            .collect::<Vec<(T, &Image)>>();

        ords.sort_by(|a, b| (a.0).partial_cmp(&b.0).unwrap());
        Some(ords.first()?.1)
    }

    fn best_image_for_width(&self, width: i32) -> Option<&Image> {
        self.best_image(|i| (width - i.width.unwrap_or(0) as i32).abs())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub images: Vec<Image>,
    pub tracks: Page<PlaylistTrack>,
    pub owner: PlaylistOwner,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PlaylistOwner {
    pub id: String,
    pub display_name: String,
}

impl WithImages for Playlist {
    fn images(&self) -> &[Image] {
        &self.images[..]
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PlaylistTrack {
    pub is_local: bool,
    pub track: FailibleTrackItem,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SavedTrack {
    pub added_at: String,
    pub track: TrackItem,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SavedAlbum {
    pub album: Album,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FullAlbum {
    #[serde(flatten)]
    pub album: Album,
    #[serde(flatten)]
    pub album_info: AlbumInfo,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Album {
    pub id: String,
    pub tracks: Option<Page<TrackItem>>,
    pub artists: Vec<Artist>,
    pub name: String,
    pub images: Vec<Image>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlbumInfo {
    pub label: String,
    pub release_date: String,
    pub copyrights: Vec<Copyright>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Copyright {
    pub text: String,
    #[serde(alias = "type")]
    pub type_: char,
}

impl WithImages for Album {
    fn images(&self) -> &[Image] {
        &self.images[..]
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Image {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub images: Option<Vec<Image>>,
}

impl WithImages for Artist {
    fn images(&self) -> &[Image] {
        if let Some(ref images) = self.images {
            images
        } else {
            &[]
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub display_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TopTracks {
    pub tracks: Vec<TrackItem>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TrackItem {
    pub id: String,
    pub uri: String,
    pub name: String,
    pub duration_ms: i64,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BadTrackItem {}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FailibleTrackItem {
    Ok(TrackItem),
    Failing(BadTrackItem),
}

impl FailibleTrackItem {
    fn get(self) -> Option<TrackItem> {
        match self {
            Self::Ok(track) => Some(track),
            Self::Failing(_) => None,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawSearchResults {
    pub albums: Option<Page<Album>>,
    pub artists: Option<Page<Artist>>,
}

impl From<Artist> for ArtistSummary {
    fn from(artist: Artist) -> Self {
        let photo = artist.best_image_for_width(200).map(|i| &i.url).cloned();
        let Artist { id, name, .. } = artist;
        Self { id, name, photo }
    }
}

impl TryFrom<PlaylistTrack> for TrackItem {
    type Error = ();

    fn try_from(PlaylistTrack { is_local, track }: PlaylistTrack) -> Result<Self, Self::Error> {
        track.get().filter(|_| !is_local).ok_or(())
    }
}

impl From<SavedTrack> for TrackItem {
    fn from(track: SavedTrack) -> Self {
        track.track
    }
}

impl From<TopTracks> for Vec<SongDescription> {
    fn from(top_tracks: TopTracks) -> Self {
        Page::new(top_tracks.tracks).into()
    }
}

impl<T> From<Page<T>> for Vec<SongDescription>
where
    T: TryInto<TrackItem>,
{
    fn from(page: Page<T>) -> Self {
        SongBatch::from(page).songs
    }
}

impl<T> From<Page<T>> for SongBatch
where
    T: TryInto<TrackItem>,
{
    fn from(page: Page<T>) -> Self {
        let batch = Batch {
            offset: page.offset(),
            batch_size: page.limit(),
            total: page.total(),
        };
        let songs = page
            .into_iter()
            .filter_map(|t| {
                let TrackItem {
                    album,
                    artists,
                    id,
                    uri,
                    name,
                    duration_ms,
                } = t.try_into().ok()?;
                let artists = artists
                    .into_iter()
                    .map(|a| ArtistRef {
                        id: a.id,
                        name: a.name,
                    })
                    .collect::<Vec<ArtistRef>>();

                let album = album.unwrap();
                let art = album.best_image_for_width(200).map(|i| &i.url).cloned();
                let Album {
                    id: album_id,
                    name: album_name,
                    ..
                } = album;
                let album_ref = AlbumRef {
                    id: album_id,
                    name: album_name,
                };

                Some(SongDescription {
                    id,
                    uri,
                    title: name,
                    artists,
                    album: album_ref,
                    duration: duration_ms as u32,
                    art,
                })
            })
            .collect();
        SongBatch { songs, batch }
    }
}

impl From<Album> for Vec<SongDescription> {
    fn from(album: Album) -> Self {
        let art = album.best_image_for_width(200).map(|i| &i.url).cloned();
        let Album { id, name, .. } = album;
        let album_ref = AlbumRef { id, name };

        album
            .tracks
            .unwrap_or_default()
            .into_iter()
            .map(|item| {
                let artists = item
                    .artists
                    .into_iter()
                    .map(|a| ArtistRef {
                        id: a.id,
                        name: a.name,
                    })
                    .collect::<Vec<ArtistRef>>();

                SongDescription {
                    id: item.id,
                    uri: item.uri,
                    title: item.name,
                    artists,
                    album: album_ref.clone(),
                    duration: item.duration_ms as u32,
                    art: art.clone(),
                }
            })
            .collect()
    }
}

impl From<FullAlbum> for AlbumFullDescription {
    fn from(full_album: FullAlbum) -> Self {
        let description = full_album.album.into();
        let release_details = full_album.album_info.into();
        Self {
            description,
            release_details,
        }
    }
}

impl From<Album> for AlbumDescription {
    fn from(album: Album) -> Self {
        let artists = album
            .artists
            .iter()
            .map(|a| ArtistRef {
                id: a.id.clone(),
                name: a.name.clone(),
            })
            .collect::<Vec<ArtistRef>>();
        let songs: Vec<SongDescription> = album.clone().into();
        let total = album.tracks.as_ref().map(|t| t.total).unwrap_or(100);
        let last_batch = Batch::first_of_size(100);
        let art = album.best_image_for_width(200).map(|i| i.url.clone());

        Self {
            id: album.id,
            title: album.name,
            artists,
            art,
            songs,
            last_batch,
            is_liked: false,
        }
    }
}

impl From<AlbumInfo> for AlbumReleaseDetails {
    fn from(info: AlbumInfo) -> Self {
        let copyrights = info
            .copyrights
            .iter()
            .map(|c| c.clone().into())
            .collect::<Vec<CopyrightDetails>>();

        Self {
            label: info.label,
            release_date: info.release_date,
            copyrights,
        }
    }
}

impl From<Copyright> for CopyrightDetails {
    fn from(copyright: Copyright) -> Self {
        Self {
            text: copyright.text,
            type_: copyright.type_,
        }
    }
}

impl From<Playlist> for PlaylistDescription {
    fn from(playlist: Playlist) -> Self {
        let art = playlist.best_image_for_width(200).map(|i| i.url.clone());
        let Playlist {
            id,
            name,
            tracks,
            owner,
            ..
        } = playlist;
        let PlaylistOwner {
            id: owner_id,
            display_name,
        } = owner;
        let song_batch = tracks.into();
        PlaylistDescription {
            id,
            title: name,
            art,
            songs: SongList::new(song_batch),
            owner: UserRef {
                id: owner_id,
                display_name,
            },
        }
    }
}
