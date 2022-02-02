use form_urlencoded::Serializer;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    convert::{Into, TryFrom, TryInto},
    vec::IntoIter,
};

use crate::app::models::*;

#[derive(Serialize)]
pub struct PlaylistDetails {
    pub name: String,
}

#[derive(Serialize)]
pub struct Uris {
    pub uris: Vec<String>,
}

#[derive(Serialize)]
pub struct PlayOffset {
    pub position: u32,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum PlayRequest {
    Contextual {
        context_uri: String,
        offset: PlayOffset,
    },
    Uris {
        uris: Vec<String>,
    },
}

#[derive(Serialize)]
pub struct Ids {
    pub ids: Vec<String>,
}

#[derive(Serialize)]
pub struct Name<'a> {
    pub name: &'a str,
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

        format!("type={types}&{serialized}")
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

    fn map<Mapper, U>(self, mapper: Mapper) -> Page<U>
    where
        Mapper: Fn(T) -> U,
    {
        let Page {
            items,
            offset,
            limit,
            total,
        } = self;
        Page {
            items: items.map(|item| item.into_iter().map(mapper).collect()),
            offset,
            limit,
            total,
        }
    }

    pub fn limit(&self) -> usize {
        self.limit
            .or_else(|| Some(self.items.as_ref()?.len()))
            .filter(|limit| *limit > 0)
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
        self.items.unwrap_or_default().into_iter()
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
    pub track: Option<FailibleTrackItem>,
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
    pub tracks: Option<Page<AlbumTrackItem>>,
    pub artists: Vec<Artist>,
    pub release_date: Option<String>,
    pub name: String,
    pub images: Vec<Image>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlbumInfo {
    pub label: String,
    pub copyrights: Vec<Copyright>,
    pub total_tracks: u32,
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
pub struct Device {
    #[serde(alias = "type")]
    pub type_: String,
    pub name: String,
    pub id: String,
    pub is_active: bool,
    pub is_restricted: bool,
    pub volume_percent: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Devices {
    pub devices: Vec<Device>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TopTracks {
    pub tracks: Vec<TrackItem>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlbumTrackItem {
    pub id: String,
    pub track_number: Option<usize>,
    pub uri: String,
    pub name: String,
    pub duration_ms: i64,
    pub artists: Vec<Artist>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TrackItem {
    #[serde(flatten)]
    pub track: AlbumTrackItem,
    pub album: Album,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BadTrackItem {}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FailibleTrackItem {
    Ok(Box<TrackItem>),
    Failing(BadTrackItem),
}

impl FailibleTrackItem {
    fn get(self) -> Option<TrackItem> {
        match self {
            Self::Ok(track) => Some(*track),
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
        track.ok_or(())?.get().filter(|_| !is_local).ok_or(())
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

impl From<(Page<AlbumTrackItem>, &Album)> for SongBatch {
    fn from(page_and_album: (Page<AlbumTrackItem>, &Album)) -> Self {
        let (page, album) = page_and_album;
        Self::from(page.map(|track| TrackItem {
            track,
            album: album.clone(),
        }))
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
                let TrackItem { track, album } = t.try_into().ok()?;
                let AlbumTrackItem {
                    artists,
                    id,
                    uri,
                    name,
                    duration_ms,
                    track_number,
                } = track;
                let artists = artists
                    .into_iter()
                    .map(|a| ArtistRef {
                        id: a.id,
                        name: a.name,
                    })
                    .collect::<Vec<ArtistRef>>();

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
                    track_number: track_number.map(|u| u as u32),
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

impl TryFrom<Album> for SongBatch {
    type Error = ();

    fn try_from(mut album: Album) -> Result<Self, Self::Error> {
        let tracks = std::mem::replace(&mut album.tracks, None).ok_or(())?;
        Ok((tracks, &album).into())
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
        let songs = album
            .clone()
            .try_into()
            .unwrap_or_else(|_| SongBatch::empty());
        let art = album.best_image_for_width(200).map(|i| i.url.clone());

        Self {
            id: album.id,
            title: album.name,
            artists,
            release_date: album.release_date,
            art,
            songs,
            is_liked: false,
        }
    }
}

impl From<AlbumInfo> for AlbumReleaseDetails {
    fn from(
        AlbumInfo {
            label,
            copyrights,
            total_tracks,
        }: AlbumInfo,
    ) -> Self {
        let copyright_text = copyrights
            .iter()
            .map(|Copyright { type_, text }| format!("[{type_}] {text}"))
            .collect::<Vec<String>>()
            .join(",\n ");

        Self {
            label,
            copyright_text,
            total_tracks: total_tracks as usize,
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
            songs: song_batch,
            owner: UserRef {
                id: owner_id,
                display_name,
            },
        }
    }
}

impl From<Device> for ConnectDevice {
    fn from(Device { id, name, .. }: Device) -> Self {
        Self { id, label: name }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_playlist_track_null() {
        let track = r#"{"is_local": false, "track": null}"#;
        let deserialized: PlaylistTrack = serde_json::from_str(track).unwrap();
        let track_item: Option<TrackItem> = deserialized.try_into().ok();
        assert!(track_item.is_none());
    }

    #[test]
    fn test_playlist_track_local() {
        let track = r#"{"is_local": true, "track": {"name": ""}}"#;
        let deserialized: PlaylistTrack = serde_json::from_str(track).unwrap();
        let track_item: Option<TrackItem> = deserialized.try_into().ok();
        assert!(track_item.is_none());
    }

    #[test]
    fn test_playlist_track_ok() {
        let track = r#"{"is_local":false,"track":{"album":{"artists":[{"external_urls":{"spotify":""},"href":"","id":"","name":"","type":"artist","uri":""}],"id":"","images":[{"height":64,"url":"","width":64}],"name":""},"artists":[{"id":"","name":""}],"duration_ms":1,"id":"","name":"","uri":""}}"#;
        let deserialized: PlaylistTrack = serde_json::from_str(track).unwrap();
        let track_item: Option<TrackItem> = deserialized.try_into().ok();
        assert!(track_item.is_some());
    }
}
