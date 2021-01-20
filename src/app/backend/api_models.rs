use regex::Regex;
use serde::Deserialize;
use std::convert::Into;

use crate::app::{AlbumDescription, AlbumRef, ArtistRef, SongDescription};

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
        let query = re.replace_all(&self.query[..], "+");

        format!(
            "q={}&type={}&offset={}&limit={}",
            query, types, self.offset, self.limit
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SavedAlbum {
    pub album: Album,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Album {
    pub id: String,
    pub tracks: Option<Tracks>,
    pub artists: Vec<Artist>,
    pub name: String,
    pub images: Vec<Image>,
}

impl Album {
    fn best_image<T: PartialOrd, F: Fn(&Image) -> T>(&self, criterion: F) -> Option<&Image> {
        let mut ords = self
            .images
            .iter()
            .map(|image| (criterion(image), image))
            .collect::<Vec<(T, &Image)>>();

        ords.sort_by(|a, b| (a.0).partial_cmp(&b.0).unwrap());
        Some(ords.first()?.1)
    }

    fn best_image_for_width(&self, width: i32) -> Option<&Image> {
        self.best_image(|i| (width - i.width as i32).abs())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Image {
    pub url: String,
    pub height: u32,
    pub width: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tracks {
    pub items: Vec<TrackItem>,
}

impl Default for Tracks {
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

impl Into<Vec<SongDescription>> for Album {
    fn into(self) -> Vec<SongDescription> {
        let art = self.best_image_for_width(200).unwrap().url.clone();
        let items = self.tracks.unwrap_or_default().items;

        let album_ref = AlbumRef {
            id: self.id.clone(),
            name: self.name.clone()
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