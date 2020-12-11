use serde::Deserialize;
use std::convert::Into;
use regex::Regex;

use crate::app::{SongDescription, AlbumDescription};

pub enum SearchType {
    Artist,
    Album
}

impl SearchType {
    fn to_string(self) -> &'static str {
        match self {
            Self::Artist => "artist",
            Self::Album => "album"
        }
    }
}

pub struct SearchQuery {
    pub query: String,
    pub types: Vec<SearchType>,
    pub limit: u32,
    pub offset: u32
}

impl SearchQuery {
    pub fn to_query_string(self) -> String {
        let mut types = self.types.into_iter().fold(
            String::new(),
            |acc, t| acc + t.to_string() + ",");
        types.pop();

        let re = Regex::new(r"(\W|\s)+").unwrap();
        let query = re.replace_all(&self.query[..], "+");

        format!("q={}&type={}&offset={}&limit={}", query, types, self.offset, self.limit)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>
}

#[derive(Deserialize, Debug, Clone)]
pub struct SavedAlbum {
    pub album: Album
}

#[derive(Deserialize, Debug, Clone)]
pub struct Album {
    pub id: String,
    pub uri: String,
    pub tracks: Option<Tracks>,
    pub artists: Vec<Artist>,
    pub name: String,
    pub images: Vec<Image>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Image {
    pub url: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Artist {
    pub uri: String,
    pub name: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tracks {
    pub items: Vec<TrackItem>
}

impl Default for Tracks {
    fn default() -> Self {
        Self { items: vec![] }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct TrackItem {
    pub uri: String,
    pub name: String,
    pub duration_ms: i64,
    pub artists: Vec<Artist>,
    pub albums: Option<Vec<Album>>
}

#[derive(Deserialize, Debug, Clone)]
pub struct SearchResults {
    pub albums: Option<Page<Album>>
}

impl Into<Vec<SongDescription>> for Album {

    fn into(self) -> Vec<SongDescription> {

        let art = self.images.first().unwrap().url.clone();
        let items = self.tracks.unwrap_or_default().items;

        items.iter().map(|item| {
            let artist = item.artists.iter()
                .map(|a| a.name.clone())
                .collect::<Vec<String>>()
                .join(", ");

            SongDescription::new(&item.name, &artist, &item.uri, item.duration_ms as u32, Some(art.clone()))
        }).collect()
    }
}

impl Into<AlbumDescription> for Album {

    fn into(self) -> AlbumDescription {

        let songs: Vec<SongDescription> = self.clone().into();
        let artist = self.artists.iter()
                .map(|a| a.name.clone())
                .collect::<Vec<String>>()
                .join(", ");
        let art = self.images.first().unwrap().url.clone();

        AlbumDescription {
            title: self.name,
            artist,
            uri: self.uri,
            art,
            songs,
            id: self.id
        }
    }
}

impl Into<SongDescription> for TrackItem {

    fn into(self) -> SongDescription {
        let art = self
            .albums.unwrap_or_default().first().unwrap()
            .images.first().unwrap()
            .url.clone();
        let artist = self.artists.iter()
            .map(|a| a.name.clone())
            .collect::<Vec<String>>()
            .join(", ");
        SongDescription::new(&self.name, &artist, &self.uri, self.duration_ms as u32, Some(art))
    }
}
