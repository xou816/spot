use isahc::prelude::*;
use isahc::HttpClientBuilder;
use serde_json::from_str;
use std::convert::Into;
use std::cell::RefCell;
use std::convert::AsRef;
use std::future::Future;
use futures::future::LocalBoxFuture;

use crate::app::{SongDescription, AlbumDescription};
use super::cache::{CacheManager, CacheFile, CachePolicy, CacheExpiry};
use super::api_models::*;
pub use super::api_models::SearchType;

const SPOTIFY_API: &'static str = "https://api.spotify.com/v1";

pub trait SpotifyApiClient {
    fn get_album(&self, id: &str) -> LocalBoxFuture<Option<AlbumDescription>>;
    fn get_track(&self, id: &str) -> LocalBoxFuture<Option<SongDescription>>;
    fn get_saved_albums(&self, offset: u32, limit: u32) -> LocalBoxFuture<Option<Vec<AlbumDescription>>>;
    fn search_albums(&self, query: &str, offset: u32, limit: u32) -> LocalBoxFuture<Option<Vec<AlbumDescription>>>;
    fn update_token(&self, token: &str);
}

#[cfg(test)]
pub mod tests {

    use super::*;

    pub struct TestSpotifyApiClient {}

    impl TestSpotifyApiClient {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl SpotifyApiClient for TestSpotifyApiClient {

        fn get_album(&self, id: &str) -> LocalBoxFuture<Option<AlbumDescription>> {
            Box::pin(async { None })
        }

        fn get_saved_albums(&self, offset: u32, limit: u32) -> LocalBoxFuture<Option<Vec<AlbumDescription>>> {
            Box::pin(async { None })
        }


        fn get_track(&self, id: &str) -> LocalBoxFuture<Option<SongDescription>> {
            Box::pin(async { None })
        }

        fn search_albums(&self, query: &str, offset: u32, limit: u32) -> LocalBoxFuture<Option<Vec<AlbumDescription>>> {
            Box::pin(async { None })
        }

        fn update_token(&self, token: &str) {}
    }
}

pub struct CachedSpotifyClient {
    token: RefCell<Option<String>>,
    client: HttpClient,
    cache: CacheManager
}

struct CacheRequest<'a, S> {
    cache: &'a CacheManager,
    resource: S,
    policy: CachePolicy
}

impl <'a, S> CacheRequest<'a, S> where S: AsRef<str> + 'a {

    fn for_resource(cache: &'a CacheManager, resource: S, policy: CachePolicy) -> Self {
        Self { cache, resource, policy }
    }

    async fn get(&self) -> Option<String> {
        match self.cache.read_cache_file(self.resource.as_ref(), self.policy).await {
            CacheFile::File(buffer) => String::from_utf8(buffer).ok(),
            _ => None
        }
    }

    async fn or_else_write<O, F>(&self, fresh: F, expiry: CacheExpiry) -> Option<String>
        where O: Future<Output=Option<String>>,  F: FnOnce() -> O {

        match self.get().await {
            Some(text) => Some(text),
            None => {
                let fresh = fresh().await?;
                self.cache.write_cache_file(
                    self.resource.as_ref(),
                    fresh.as_bytes(),
                    expiry).await?;
                Some(fresh)
            }
        }
    }
}

impl CachedSpotifyClient {

    pub fn new() -> CachedSpotifyClient {
        let client = HttpClient::builder()
            .ssl_options(isahc::config::SslOption::DANGER_ACCEPT_INVALID_CERTS)
            .build().unwrap();
        CachedSpotifyClient { token: RefCell::new(None), client, cache: CacheManager::new() }
    }

    fn default_cache_policy(&self) -> CachePolicy {
        match *self.token.borrow() {
            Some(_) => CachePolicy::Default,
            None => CachePolicy::IgnoreExpiry
        }
    }

    fn cache_request<'a, S: AsRef<str> + 'a>(&'a self, resource: S, policy: Option<CachePolicy>) -> CacheRequest<'a, S> {
        CacheRequest::for_resource(&self.cache, resource, policy.unwrap_or(self.default_cache_policy()))
    }

    async fn get_album_no_cache(&self, id: &str) -> Option<String> {

        let token = self.token.borrow();
        let token = token.as_deref()?;

        let uri = format!("{}/albums/{}", SPOTIFY_API, id);
        let request = Request::get(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }

    async fn get_saved_albums_no_cache(&self, offset: u32, limit: u32) -> Option<String> {

        let token = self.token.borrow();
        let token = token.as_deref()?;

        let uri = format!("{}/me/albums?offset={}&limit={}", SPOTIFY_API, offset, limit);
        let request = Request::get(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }

    async fn get_track_no_cache(&self, id: &str) -> Option<String> {

        let token = self.token.borrow();
        let token = token.as_deref()?;

        let uri = format!("{}/tracks/{}", SPOTIFY_API, id);
        let request = Request::get(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }

    async fn search_no_cache(&self, query: String) -> Option<String> {
        let token = self.token.borrow();
        let token = token.as_deref()?;

        let query = SearchQuery { query, types: vec![SearchType::Album], limit: 5, offset: 0 };
        let uri = format!("{}/search?{}", SPOTIFY_API, query.to_query_string());

        let request = Request::get(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }
}

impl SpotifyApiClient for CachedSpotifyClient {

    fn update_token(&self, token: &str) {
        self.token.replace(Some(token.to_string()));
    }

    fn get_track(&self, id: &str) -> LocalBoxFuture<Option<SongDescription>> {
        let id = id.to_owned();

        Box::pin(async move {

            let cache_request = self.cache_request(
                format!("track_{}.json", id),
                None);

            let text = cache_request
                .or_else_write(
                    || self.get_track_no_cache(&id[..]),
                    CacheExpiry::expire_in_seconds(3600))
                .await?;

            let track = from_str::<TrackItem>(&text).ok()?;

            Some(track.into())
        })
    }

    fn get_saved_albums(&self, offset: u32, limit: u32) -> LocalBoxFuture<Option<Vec<AlbumDescription>>> {
        Box::pin(async move {

            let cache_request = self.cache_request(
                format!("me_albums_{}_{}.json", offset, limit),
                None);

            let text = cache_request
                .or_else_write(
                    || self.get_saved_albums_no_cache(offset, limit),
                    CacheExpiry::expire_in_seconds(3600))
                .await?;

            let page = from_str::<Page<SavedAlbum>>(&text).ok()?;

            let albums = page.items.into_iter()
                .map(|saved| saved.album.into())
                .collect::<Vec<AlbumDescription>>();

            Some(albums)
        })
    }


    fn get_album(&self, id: &str) -> LocalBoxFuture<Option<AlbumDescription>> {

        let id = id.to_owned();

        Box::pin(async move {

            let cache_request = self.cache_request(
                format!("album_{}.json", id),
                None);

            let text = cache_request
                .or_else_write(
                    || self.get_album_no_cache(&id[..]),
                    CacheExpiry::expire_in_seconds(3600))
                .await?;

            Some(from_str::<Album>(&text).ok()?.into())
        })
    }

    fn search_albums(&self, query: &str, offset: u32, limit: u32) -> LocalBoxFuture<Option<Vec<AlbumDescription>>> {

        let query = query.to_owned();

        Box::pin(async move {
            let text = self.search_no_cache(query).await?;

            let results = from_str::<SearchResults>(&text).ok()?;

            Some(results.albums?.items.into_iter()
                .map(|saved| saved.into())
                .collect::<Vec<AlbumDescription>>())

        })
    }
}
