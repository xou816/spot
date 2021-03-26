use form_urlencoded::Serializer;
use regex::Regex;
use serde::Deserialize;
use std::convert::Into;

use crate::app::models::*;

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
    pub limit: u32,
    pub offset: u32,
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
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { items: Some(items) }
    }

    pub fn empty() -> Self {
        Self { items: None }
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

impl Into<ArtistSummary> for Artist {
    fn into(self) -> ArtistSummary {
        let photo = self.best_image_for_width(200).map(|i| &i.url).cloned();
        let Artist { id, name, .. } = self;
        ArtistSummary { id, name, photo }
    }
}

impl Into<Vec<SongDescription>> for Page<PlaylistTrack> {
    fn into(self) -> Vec<SongDescription> {
        let items = self
            .into_iter()
            .filter_map(|PlaylistTrack { is_local, track }| track.get().filter(|_| !is_local))
            .collect::<Vec<TrackItem>>();
        Page::new(items).into()
    }
}

impl Into<Vec<SongDescription>> for TopTracks {
    fn into(self) -> Vec<SongDescription> {
        Page::new(self.tracks).into()
    }
}

impl Into<Vec<SongDescription>> for Page<TrackItem> {
    fn into(self) -> Vec<SongDescription> {
        self.into_iter()
            .map(
                |TrackItem {
                     album,
                     artists,
                     id,
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

impl Into<Vec<SongDescription>> for Album {
    fn into(self) -> Vec<SongDescription> {
        let art = self.best_image_for_width(200).map(|i| &i.url).cloned();
        let Album { id, name, .. } = self;
        let album_ref = AlbumRef { id, name };

        self.tracks
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

impl Into<AlbumDescription> for Album {
    fn into(self) -> AlbumDescription {
        let artists = self
            .artists
            .iter()
            .map(|a| ArtistRef {
                id: a.id.clone(),
                name: a.name.clone(),
            })
            .collect::<Vec<ArtistRef>>();
        let songs: Vec<SongDescription> = self.clone().into();
        let art = self.best_image_for_width(200).map(|i| i.url.clone());

        AlbumDescription {
            id: self.id,
            title: self.name,
            artists,
            art,
            songs,
            is_liked: false,
        }
    }
}

impl Into<PlaylistDescription> for Playlist {
    fn into(self) -> PlaylistDescription {
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
        PlaylistDescription {
            id,
            title: name,
            art,
            songs: tracks.into(),
            owner: UserRef {
                id: owner_id,
                display_name,
            },
        }
    }
}
