use isahc::prelude::*;
use serde::Deserialize;
use std::convert::Into;

use crate::app::{SongDescription, AlbumDescription};

#[derive(Deserialize, Debug, Clone)]
pub struct Page<T> {
    items: Vec<T>
}

#[derive(Deserialize, Debug, Clone)]
pub struct SavedAlbum {
    album: Album
}

#[derive(Deserialize, Debug, Clone)]
pub struct Album {
    id: String,
    uri: String,
    tracks: Tracks,
    artists: Vec<Artist>,
    name: String,
    images: Vec<Image>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Image {
    url: String
}

#[derive(Deserialize, Debug, Clone)]
struct Artist {
    uri: String,
    name: String
}

#[derive(Deserialize, Debug, Clone)]
struct Tracks {
    items: Vec<TrackItem>
}

#[derive(Deserialize, Debug, Clone)]
struct TrackItem {
    uri: String,
    name: String,
    duration_ms: i64,
    artists: Vec<Artist>
}

impl Into<Vec<SongDescription>> for Album {
    fn into(self) -> Vec<SongDescription> {
        self.tracks.items.iter().map(|item| {
            let artist = item.artists.iter()
                .map(|a| a.name.clone())
                .collect::<Vec<String>>()
                .join(", ");

            SongDescription::new(&item.name, &artist, &item.uri, item.duration_ms as u32)
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

const SPOTIFY_API: &'static str = "https://api.spotify.com/v1";

pub struct SpotifyApi {

}

impl SpotifyApi {

    pub fn new() -> SpotifyApi {
        SpotifyApi {}
    }

    pub async fn get_album(&self, token: &str, id: &str) -> Option<Vec<SongDescription>> {
        let uri = format!("{}/albums/{}", SPOTIFY_API, id);
        println!("{}", uri);
        let request = Request::get(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();
        let result = request.send_async().await;

        result.ok()?.json::<Album>().ok().map(|album| album.into())
    }

    pub async fn get_saved_albums(&self, token: &str) -> Option<Vec<AlbumDescription>> {
        let uri = format!("{}/me/albums", SPOTIFY_API);
        println!("{}", uri);
        let request = Request::get(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();
        let result = request.send_async().await;

        let page = result.ok()?.json::<Page<SavedAlbum>>().ok()?;

        Some(page.items.iter()
            .map(|saved| saved.album.clone().into())
            .collect::<Vec<AlbumDescription>>())
    }
}

