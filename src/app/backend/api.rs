use form_urlencoded::Serializer;
use futures::future::BoxFuture;
use isahc::config::Configurable;
use isahc::http::{StatusCode, Uri};
use isahc::{AsyncReadResponseExt, HttpClient, Request};
use serde_json::from_str;
use std::convert::{AsRef, Into};
use std::sync::Mutex;
use thiserror::Error;

pub use super::api_models::SearchType;
use super::api_models::*;
use super::cache::{CacheExpiry, CacheManager, CachePolicy, CacheRequest};
use crate::app::credentials::Credentials;
use crate::app::models::*;

const SPOTIFY_HOST: &str = "api.spotify.com";

pub type SpotifyResult<T> = Result<T, SpotifyApiError>;

pub trait SpotifyApiClient {
    fn get_artist(&self, id: &str) -> BoxFuture<SpotifyResult<ArtistDescription>>;
    fn get_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>>;
    fn get_saved_albums(
        &self,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>>;
    fn search_albums(
        &self,
        query: &str,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>>;
    fn update_credentials(&self, credentials: Credentials);
    fn update_token(&self, token: String);
}

#[derive(Error, Debug)]
pub enum SpotifyApiError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("No token")]
    NoToken,
    #[error(transparent)]
    ClientError(#[from] isahc::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),
}

pub struct CachedSpotifyClient {
    credentials: Mutex<Option<Credentials>>,
    client: HttpClient,
    cache: CacheManager,
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
            policy.unwrap_or_else(|| self.default_cache_policy()),
        )
    }

    fn uri(&self, path: String, query: Option<String>) -> Result<Uri, isahc::http::Error> {
        let path_and_query = match query {
            None => path,
            Some(query) => format!("{}?{}", path, query),
        };
        Uri::builder()
            .scheme("https")
            .authority(SPOTIFY_HOST)
            .path_and_query(&path_and_query[..])
            .build()
    }

    fn make_query_params() -> Serializer<'static, String> {
        Serializer::new(String::new())
    }

    async fn send<T>(&self, req: Request<T>) -> Result<String, SpotifyApiError>
    where
        T: Into<isahc::AsyncBody>,
    {
        let mut result = self.client.send_async(req).await?;
        match result.status() {
            StatusCode::UNAUTHORIZED => Err(SpotifyApiError::InvalidToken),
            _ => Ok(result.text().await?),
        }
    }
}

impl CachedSpotifyClient {
    async fn get_artist_no_cache(&self, id: &str) -> Result<String, SpotifyApiError> {
        let request = {
            let creds = self.credentials.lock().unwrap();
            let creds = creds.as_ref().ok_or(SpotifyApiError::NoToken)?;
            let uri = self.uri(format!("/v1/artists/{}", id), None).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        self.send(request).await
    }

    async fn get_artist_albums_no_cache(&self, id: &str) -> Result<String, SpotifyApiError> {
        let request = {
            let creds = self.credentials.lock().unwrap();
            let creds = creds.as_ref().ok_or(SpotifyApiError::NoToken)?;

            let query = Self::make_query_params()
                .append_pair("include_groups", "album")
                .append_pair("country", "from_token")
                .finish();

            let uri = self
                .uri(format!("/v1/artists/{}/albums", id), Some(query))
                .unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        self.send(request).await
    }

    async fn get_album_no_cache(&self, id: &str) -> Result<String, SpotifyApiError> {
        let request = {
            let creds = self.credentials.lock().unwrap();
            let creds = creds.as_ref().ok_or(SpotifyApiError::NoToken)?;

            let uri = self.uri(format!("/v1/albums/{}", id), None).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        self.send(request).await
    }

    async fn get_saved_albums_no_cache(
        &self,
        offset: u32,
        limit: u32,
    ) -> Result<String, SpotifyApiError> {
        let request = {
            let creds = self.credentials.lock().unwrap();
            let creds = creds.as_ref().ok_or(SpotifyApiError::NoToken)?;

            let query = Self::make_query_params()
                .append_pair("offset", &offset.to_string()[..])
                .append_pair("limit", &limit.to_string()[..])
                .finish();

            let uri = self.uri("/v1/me/albums".to_string(), Some(query)).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        self.send(request).await
    }

    async fn search_no_cache(&self, query: String) -> Result<String, SpotifyApiError> {
        let request = {
            let creds = self.credentials.lock().unwrap();
            let creds = creds.as_ref().ok_or(SpotifyApiError::NoToken)?;

            let query = SearchQuery {
                query,
                types: vec![SearchType::Album],
                limit: 5,
                offset: 0,
            };

            let uri = self
                .uri("/v1/search".to_string(), Some(query.into_query_string()))
                .unwrap();

            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &creds.token))
                .body(())
                .unwrap()
        };

        self.send(request).await
    }
}

impl SpotifyApiClient for CachedSpotifyClient {
    fn update_credentials(&self, credentials: Credentials) {
        if let Ok(mut mut_creds) = self.credentials.lock() {
            *mut_creds = Some(credentials);
        }
    }

    fn update_token(&self, token: String) {
        if let Ok(mut mut_creds) = self.credentials.lock() {
            if let Some(mut mut_creds) = mut_creds.as_mut() {
                (*mut_creds).token = token;
            }
        }
    }


    fn get_saved_albums(
        &self,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>> {
        Box::pin(async move {
            let cache_request =
                self.cache_request(format!("net/me_albums_{}_{}.json", offset, limit), None);

            let text = cache_request
                .or_else_try_write(
                    || self.get_saved_albums_no_cache(offset, limit),
                    CacheExpiry::expire_in_seconds(3600),
                )
                .await?;

            let page = from_str::<Page<SavedAlbum>>(&text)?;

            let albums = page
                .items
                .into_iter()
                .map(|saved| saved.album.into())
                .collect::<Vec<AlbumDescription>>();

            Ok(albums)
        })
    }

    fn get_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let cache_request = self.cache_request(format!("net/album_{}.json", id), None);

            let text = cache_request
                .or_else_try_write(
                    || self.get_album_no_cache(&id[..]),
                    CacheExpiry::expire_in_hours(24),
                )
                .await?;

            Ok(from_str::<Album>(&text)?.into())
        })
    }

    fn get_artist(&self, id: &str) -> BoxFuture<Result<ArtistDescription, SpotifyApiError>> {
        let id = id.to_owned();

        Box::pin(async move {
            let req = self.cache_request(format!("net/artist_{}.json", id), None);
            let artist = req.or_else_try_write(
                || self.get_artist_no_cache(&id[..]),
                CacheExpiry::expire_in_hours(24),
            );

            let req = self.cache_request(format!("net/artist_albums_{}.json", id), None);
            let albums = req.or_else_try_write(
                || self.get_artist_albums_no_cache(&id[..]),
                CacheExpiry::expire_in_hours(24),
            );

            let artist: Artist = from_str(&artist.await?)?;
            let albums: Page<Album> = from_str(&albums.await?)?;

            let albums = albums
                .items
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<AlbumDescription>>();

            let result = ArtistDescription {
                name: artist.name,
                albums,
            };
            Ok(result)
        })
    }

    fn search_albums(
        &self,
        query: &str,
        _offset: u32,
        _limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>> {

        let query = query.to_owned();

        Box::pin(async move {
            let text = self.search_no_cache(query).await?;

            let results = from_str::<SearchResults>(&text)?;

            match results.albums {
                Some(albums) => Ok(albums
                    .items
                    .into_iter()
                    .map(|saved| saved.into())
                    .collect::<Vec<AlbumDescription>>()),
                None => Ok(vec![]),
            }
        })
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_search_query() {
        let query = SearchQuery {
            query: "test".to_string(),
            types: vec![SearchType::Album, SearchType::Artist],
            limit: 5,
            offset: 0,
        };

        assert_eq!(
            query.into_query_string(),
            "type=album,artist&q=test&offset=0&limit=5&market=from_token"
        );
    }

    #[test]
    fn test_search_query_spaces_and_stuff() {
        let query = SearchQuery {
            query: "test??? wow".to_string(),
            types: vec![SearchType::Album],
            limit: 5,
            offset: 0,
        };

        assert_eq!(
            query.into_query_string(),
            "type=album&q=test+wow&offset=0&limit=5&market=from_token"
        );
    }

    #[test]
    fn test_search_query_encoding() {
        let query = SearchQuery {
            query: "кириллица".to_string(),
            types: vec![SearchType::Album],
            limit: 5,
            offset: 0,
        };

        assert_eq!(query.into_query_string(), "type=album&q=%D0%BA%D0%B8%D1%80%D0%B8%D0%BB%D0%BB%D0%B8%D1%86%D0%B0&offset=0&limit=5&market=from_token");
    }
}
