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
    pub items: Vec<T>,
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
pub struct DetailedPlaylist {
    pub id: String,
    pub name: String,
    pub images: Vec<Image>,
    pub tracks: Tracks<PlaylistTrack>,
    pub owner: PlaylistOwner,
}

impl WithImages for DetailedPlaylist {
    fn images(&self) -> &[Image] {
        &self.images[..]
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PlaylistTrack {
    pub is_local: bool,
    pub track: TrackItem,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SavedAlbum {
    pub album: Album,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Album {
    pub id: String,
    pub tracks: Option<Tracks<TrackItem>>,
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
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tracks<Item> {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TopTracks {
    pub tracks: Vec<TrackItem>,
}

impl<Item> Default for Tracks<Item> {
    fn default() -> Self {
        Self { items: vec![] }
    }
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
pub struct SearchResults {
    pub albums: Option<Page<Album>>,
}

impl Into<Vec<SongDescription>> for DetailedPlaylist {
    fn into(self) -> Vec<SongDescription> {
        let items = self
            .tracks
            .items
            .into_iter()
            .filter(|t| !t.is_local)
            .map(|item| item.track)
            .collect::<Vec<TrackItem>>();
        Tracks { items }.into()
    }
}

impl Into<Vec<SongDescription>> for TopTracks {
    fn into(self) -> Vec<SongDescription> {
        Tracks { items: self.tracks }.into()
    }
}

impl Into<Vec<SongDescription>> for Tracks<TrackItem> {
    fn into(self) -> Vec<SongDescription> {
        self.items
            .into_iter()
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
                    let art = album.best_image_for_width(200).unwrap().url.clone();
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
                        art: Some(art),
                    }
                },
            )
            .collect()
    }
}

impl Into<Vec<SongDescription>> for Album {
    fn into(self) -> Vec<SongDescription> {
        let art = self.best_image_for_width(200).unwrap().url.clone();
        let items = self.tracks.unwrap_or_default().items;

        let album_ref = AlbumRef {
            id: self.id.clone(),
            name: self.name.clone(),
        };

        items
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
                    art: Some(art.clone()),
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
        let art = self.best_image_for_width(200).unwrap().url.clone();

        AlbumDescription {
            id: self.id,
            title: self.name,
            artists,
            art,
            songs,
        }
    }
}

impl Into<PlaylistDescription> for Playlist {
    fn into(self) -> PlaylistDescription {
        let art = self.best_image_for_width(200).unwrap().url.clone();
        let PlaylistOwner { id, display_name } = self.owner;
        PlaylistDescription {
            id: self.id,
            title: self.name,
            art,
            songs: vec![],
            owner: UserRef { id, display_name },
        }
    }
}

impl Into<PlaylistDescription> for DetailedPlaylist {
    fn into(self) -> PlaylistDescription {
        let songs: Vec<SongDescription> = self.clone().into();
        let art = self.best_image_for_width(200).unwrap().url.clone();
        let PlaylistOwner { id, display_name } = self.owner;
        PlaylistDescription {
            id: self.id,
            title: self.name,
            art,
            songs,
            owner: UserRef { id, display_name },
        }
    }
}
