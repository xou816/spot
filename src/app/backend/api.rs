use isahc::prelude::*;
use serde::Deserialize;
use std::convert::Into;

use crate::app::SongDescription;

#[derive(Deserialize, Debug)]
pub struct Album {
    tracks: Tracks,
    artists: Vec<Artist>
}

#[derive(Deserialize, Debug)]
struct Artist {
    uri: String,
    name: String
}

#[derive(Deserialize, Debug)]
struct Tracks {
    items: Vec<TrackItem>
}

#[derive(Deserialize, Debug)]
struct TrackItem {
    uri: String,
    name: String,
    duration_ms: i64,
    artists: Vec<Artist>
}

impl Into<Vec<SongDescription>> for Album {
    fn into(self) -> Vec<SongDescription> {
        self.tracks.items.iter().map(|item| {
            let artist = item.artists.iter().map(|a| a.name.clone()).collect::<Vec<String>>().join(", ");
            SongDescription::new(&item.name, &artist, &item.uri)
        }).collect()
    }
}

const SPOTIFY_API: &'static str = "https://api.spotify.com/v1";

pub async fn get_album(token: String, id: &str) -> Option<Vec<SongDescription>> {
    let uri = format!("{}/albums/{}", SPOTIFY_API, id);
    let request = Request::get(uri)
        .header("Authorization", format!("Bearer {}", &token))
        .body(())
        .unwrap();
    let result = request.send_async().await;

    result.ok()?.json::<Album>().ok().map(|album| album.into())
}

