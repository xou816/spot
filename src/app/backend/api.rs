use futures::future::BoxFuture;
use isahc::prelude::*;
use serde_json::from_str;
use std::convert::AsRef;
use std::convert::Into;
use std::future::Future;
use std::sync::Mutex;

pub use super::api_models::SearchType;
use super::api_models::*;
use super::cache::{CacheExpiry, CacheFile, CacheManager, CachePolicy};
use crate::app::credentials::Credentials;
use crate::app::models::*;

const SPOTIFY_API: &'static str = "https://api.spotify.com/v1";

pub trait SpotifyApiClient {
    fn get_artist(&self, id: &str) -> BoxFuture<Option<ArtistDescription>>;
    fn get_album(&self, id: &str) -> BoxFuture<Option<AlbumDescription>>;
    fn get_saved_albums(&self, offset: u32, limit: u32)
        -> BoxFuture<Option<Vec<AlbumDescription>>>;
    fn search_albums(
        &self,
        query: &str,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<Option<Vec<AlbumDescription>>>;
    fn update_credentials(&self, credentials: Credentials);
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
        fn get_artist(&self, id: &str) -> BoxFuture<Option<ArtistDescription>> {
            Box::pin(async { None })
        }

        fn get_album(&self, id: &str) -> BoxFuture<Option<AlbumDescription>> {
            Box::pin(async { None })
        }

        fn get_saved_albums(
            &self,
            offset: u32,
            limit: u32,
        ) -> BoxFuture<Option<Vec<AlbumDescription>>> {
            Box::pin(async { None })
        }

        fn search_albums(
            &self,
            query: &str,
            offset: u32,
            limit: u32,
        ) -> BoxFuture<Option<Vec<AlbumDescription>>> {
            Box::pin(async { None })
        }

        fn update_credentials(&self, credentials: Credentials) {}
    }
}

pub struct CachedSpotifyClient {
    credentials: Mutex<Option<Credentials>>,
    client: HttpClient,
    cache: CacheManager,
}

struct CacheRequest<'a, S> {
    cache: &'a CacheManager,
    resource: S,
    policy: CachePolicy,
}

impl<'a, S> CacheRequest<'a, S>
where
    S: AsRef<str> + 'a,
{
    fn for_resource(cache: &'a CacheManager, resource: S, policy: CachePolicy) -> Self {
        Self {
            cache,
            resource,
            policy,
        }
    }

    async fn get(&self) -> Option<String> {
        match self
            .cache
            .read_cache_file(self.resource.as_ref(), self.policy)
            .await
        {
            CacheFile::File(buffer) => String::from_utf8(buffer).ok(),
            _ => None,
        }
    }

    async fn or_else_write<O, F>(&self, fresh: F, expiry: CacheExpiry) -> Option<String>
    where
        O: Future<Output = Option<String>>,
        F: FnOnce() -> O,
    {
        match self.get().await {
            Some(text) => Some(text),
            None => {
                let fresh = fresh().await?;
                self.cache
                    .write_cache_file(self.resource.as_ref(), fresh.as_bytes(), expiry)
                    .await?;
                Some(fresh)
            }
        }
    }
}

impl CachedSpotifyClient {
    pub fn new() -> CachedSpotifyClient {
        let mut builder = HttpClient::builder();
        if cfg!(debug_assertions) {
            builder = builder.ssl_options(isahc::config::SslOption::DANGER_ACCEPT_INVALID_CERTS);
        }
        let client = builder.build().unwrap();
        CachedSpotifyClient {
            credentials: Mutex::new(None),
            client,
            cache: CacheManager::new(&["net"]).unwrap(),
        }
    }

    fn default_cache_policy(&self) -> CachePolicy {
        match *self.credentials.lock().unwrap() {
            Some(_) => CachePolicy::Default,
            None => CachePolicy::IgnoreExpiry,
        }
    }

    fn cache_request<'a, S: AsRef<str> + 'a>(
        &'a self,
        resource: S,
        policy: Option<CachePolicy>,
    ) -> CacheRequest<'a, S> {
        CacheRequest::for_resource(
            &self.cache,
            resource,
            policy.unwrap_or(self.default_cache_policy()),
        )
    }

    async fn get_artist_no_cache(&self, id: &str) -> Option<String> {
        let request = {
            let creds = self.credentials.lock().ok()?;
            let creds = creds.as_ref()?;

            let uri = format!("{}/artists/{}", SPOTIFY_API, id);
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }

    async fn get_artist_albums_no_cache(&self, id: &str) -> Option<String> {
        let request = {
            let creds = self.credentials.lock().ok()?;
            let creds = creds.as_ref()?;

            let uri = format!(
                "{}/artists/{}/albums?include_groups=album&country=from_token",
                SPOTIFY_API, id
            );
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }

    async fn get_album_no_cache(&self, id: &str) -> Option<String> {
        let request = {
            let creds = self.credentials.lock().ok()?;
            let creds = creds.as_ref()?;

            let uri = format!("{}/albums/{}", SPOTIFY_API, id);
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }

    async fn get_saved_albums_no_cache(&self, offset: u32, limit: u32) -> Option<String> {
        let request = {
            let creds = self.credentials.lock().ok()?;
            let creds = creds.as_ref()?;

            let uri = format!(
                "{}/me/albums?offset={}&limit={}",
                SPOTIFY_API, offset, limit
            );
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }

    async fn search_no_cache(&self, query: String) -> Option<String> {
        let request = {
            let creds = self.credentials.lock().ok()?;
            let token = &creds.as_ref()?.token;

            let query = SearchQuery {
                query,
                types: vec![SearchType::Album],
                limit: 5,
                offset: 0,
            };
            let uri = format!(
                "{}/search?{}&market=from_token",
                SPOTIFY_API,
                query.to_query_string()
            );

            Request::get(uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(())
                .unwrap()
        };

        let mut result = self.client.send_async(request).await.ok()?;
        result.text_async().await.ok()
    }
}

impl SpotifyApiClient for CachedSpotifyClient {
    fn update_credentials(&self, credentials: Credentials) {
        if let Some(ref mut mut_creds) = self.credentials.lock().ok() {
            **mut_creds = Some(credentials);
        }
    }

    fn get_saved_albums(
        &self,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<Option<Vec<AlbumDescription>>> {
        Box::pin(async move {
            let cache_request =
                self.cache_request(format!("net/me_albums_{}_{}.json", offset, limit), None);

            let text = cache_request
                .or_else_write(
                    || self.get_saved_albums_no_cache(offset, limit),
                    CacheExpiry::expire_in_seconds(3600),
                )
                .await?;

            let page = from_str::<Page<SavedAlbum>>(&text).ok()?;

            let albums = page
                .items
                .into_iter()
                .map(|saved| saved.album.into())
                .collect::<Vec<AlbumDescription>>();

            Some(albums)
        })
    }

    fn get_album(&self, id: &str) -> BoxFuture<Option<AlbumDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let cache_request = self.cache_request(format!("net/album_{}.json", id), None);

            let text = cache_request
                .or_else_write(
                    || self.get_album_no_cache(&id[..]),
                    CacheExpiry::expire_in_hours(24),
                )
                .await?;

            Some(from_str::<Album>(&text).ok()?.into())
        })
    }

    fn get_artist(&self, id: &str) -> BoxFuture<Option<ArtistDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let req = self.cache_request(format!("net/artist_{}.json", id), None);
            let artist = req.or_else_write(
                || self.get_artist_no_cache(&id[..]),
                CacheExpiry::expire_in_hours(24),
            );

            let req = self.cache_request(format!("net/artist_albums_{}.json", id), None);
            let albums = req.or_else_write(
                || self.get_artist_albums_no_cache(&id[..]),
                CacheExpiry::expire_in_hours(24),
            );

            let artist: Artist = from_str(&artist.await?).ok()?;
            let albums: Page<Album> = from_str(&albums.await?).ok()?;

            let albums = albums
                .items
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<AlbumDescription>>();

            let result = ArtistDescription {
                name: artist.name,
                albums,
            };
            Some(result)
        })
    }

    fn search_albums(
        &self,
        query: &str,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<Option<Vec<AlbumDescription>>> {
        let query = query.to_owned();

        Box::pin(async move {
            let text = self.search_no_cache(query).await?;

            let results = from_str::<SearchResults>(&text).ok()?;

            Some(
                results
                    .albums?
                    .items
                    .into_iter()
                    .map(|saved| saved.into())
                    .collect::<Vec<AlbumDescription>>(),
            )
        })
    }
}
