use form_urlencoded::Serializer;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::convert::Into;

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
    pub total: usize,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            total: items.len(),
            items: Some(items),
        }
    }

    pub fn empty() -> Self {
        Self {
            items: None,
            total: 0,
        }
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.unwrap_or_else(Vec::new).into_iter()
    }
}

impl<T> Default for Page<T> {
    fn default() -> Self {
        Self::empty()
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
pub struct SavedAlbum {
    pub album: Album,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Album {
    pub id: String,
    pub tracks: Option<Page<TrackItem>>,
    pub artists: Vec<Artist>,
    pub name: String,
    pub label: Option<String>,
    pub release_date: Option<String>,
    pub copyrights: Option<Vec<Copyright>>,
    pub images: Vec<Image>,
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
pub struct Copyright {
    pub text: String,
    #[serde(alias = "type")]
    pub type_: char,
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

impl From<Page<PlaylistTrack>> for Vec<SongDescription> {
    fn from(page: Page<PlaylistTrack>) -> Self {
        let items = page
            .into_iter()
            .filter_map(|PlaylistTrack { is_local, track }| track.get().filter(|_| !is_local))
            .collect::<Vec<TrackItem>>();
        Page::new(items).into()
    }
}

impl From<TopTracks> for Vec<SongDescription> {
    fn from(top_tracks: TopTracks) -> Self {
        Page::new(top_tracks.tracks).into()
    }
}

impl From<Page<TrackItem>> for Vec<SongDescription> {
    fn from(page: Page<TrackItem>) -> Self {
        page.into_iter()
            .map(
                |TrackItem {
                     album,
                     artists,
                     id,
                     uri,
                     name,
                     duration_ms,
                 }| {
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

                    SongDescription {
                        id,
                        uri,
                        title: name,
                        artists,
                        album: album_ref,
                        duration: duration_ms as u32,
                        art,
                    }
                },
            )
            .collect()
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
        let art = album.best_image_for_width(200).map(|i| i.url.clone());
        let mut copyrights = vec![];
        if let Some(c) = album.copyrights {
            copyrights = c
                .iter()
                .map(|c| CopyrightRef {
                    text: c.text.clone(),
                    type_: c.type_,
                })
                .collect::<Vec<CopyrightRef>>();
        };

        Self {
            id: album.id,
            title: album.name,
            artists,
            art,
            songs,
            label: album
                .label
                .unwrap_or_else(|| "No label provided".to_owned()),
            release_date: album
                .release_date
                .unwrap_or_else(|| "No release date provided".to_owned()),
            copyrights,
            is_liked: false,
        }
    }
}

impl Playlist {
    pub fn into_playlist_description(
        self,
        batch_size: usize,
        offset: usize,
    ) -> PlaylistDescription {
        let art = self.best_image_for_width(200).map(|i| i.url.clone());
        let Playlist {
            id,
            name,
            tracks,
            owner,
            ..
        } = self;
        let PlaylistOwner {
            id: owner_id,
            display_name,
        } = owner;
        let total = tracks.total;
        PlaylistDescription {
            id,
            title: name,
            art,
            songs: tracks.into(),
            last_batch: Batch {
                offset,
                batch_size,
                total,
            },
            owner: UserRef {
                id: owner_id,
                display_name,
            },
        }
    }
}
