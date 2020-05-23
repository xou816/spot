use isahc::prelude::*;
use serde_json::Value;

pub async fn get_playlist(token: &str, playlist: &str) -> Option<Value> {
    let uri = format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist);
    let request = Request::get(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(())
        .unwrap();
    let result = request.send_async().await;
    result.ok().and_then(|mut response| {
        response.json::<Value>().ok()
    })
}
