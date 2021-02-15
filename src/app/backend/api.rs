use form_urlencoded::Serializer;
use futures::future::BoxFuture;
use isahc::config::Configurable;
use isahc::http::{StatusCode, Uri};
use isahc::{AsyncReadResponseExt, HttpClient, Request};
use regex::Regex;
use serde_json::from_str;
use std::convert::{AsRef, Into};
use std::sync::Mutex;
use thiserror::Error;

pub use super::api_models::SearchType;
use super::api_models::*;
use super::cache::{CacheExpiry, CacheManager, CachePolicy, CacheRequest};
use crate::app::models::*;

lazy_static! {
    static ref ME_ALBUMS_CACHE: Regex = Regex::new(r"^me_albums_\w+_\w+\.json.expiry$").unwrap();
}

const SPOTIFY_HOST: &str = "api.spotify.com";

pub type SpotifyResult<T> = Result<T, SpotifyApiError>;

pub trait SpotifyApiClient {
    fn get_artist(&self, id: &str) -> BoxFuture<SpotifyResult<ArtistDescription>>;

    fn get_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>>;

    fn get_playlist(&self, id: &str) -> BoxFuture<SpotifyResult<PlaylistDescription>>;

    fn get_saved_albums(
        &self,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>>;

    fn save_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>>;
    fn remove_saved_album(&self, id: &str) -> BoxFuture<SpotifyResult<()>>;

    fn get_saved_playlists(
        &self,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<PlaylistDescription>>>;

    fn search(
        &self,
        query: &str,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<SearchResults>>;

    fn get_artist_albums(
        &self,
        id: &str,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>>;

    fn update_token(&self, token: String);
}

#[derive(Error, Debug)]
pub enum SpotifyApiError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("No token")]
    NoToken,
    #[error("Request failed with status {0}")]
    BadStatus(u16),
    #[error(transparent)]
    ClientError(#[from] isahc::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),
}

pub struct CachedSpotifyClient {
    token: Mutex<Option<String>>,
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
            token: Mutex::new(None),
            client,
            cache: CacheManager::new(&["net"]).unwrap(),
        }
    }

    fn default_cache_policy(&self) -> CachePolicy {
        match *self.token.lock().unwrap() {
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

    async fn send_req<B, F>(&self, make_request: F) -> Result<String, SpotifyApiError>
    where
        B: Into<isahc::AsyncBody>,
        F: FnOnce(&String) -> Request<B>,
    {
        let request = {
            let token = self.token.lock().unwrap();
            let token = token.as_ref().ok_or(SpotifyApiError::NoToken)?;
            make_request(token)
        };

        let mut result = self.client.send_async(request).await?;
        match result.status() {
            StatusCode::UNAUTHORIZED => Err(SpotifyApiError::InvalidToken),
            s if s.is_success() => Ok(result.text().await?),
            s => Err(SpotifyApiError::BadStatus(s.as_u16())),
        }
    }

    async fn send_req_no_response<B, F>(&self, make_request: F) -> Result<(), SpotifyApiError>
    where
        B: Into<isahc::AsyncBody>,
        F: FnOnce(&String) -> Request<B>,
    {
        let request = {
            let token = self.token.lock().unwrap();
            let token = token.as_ref().ok_or(SpotifyApiError::NoToken)?;
            make_request(token)
        };

        let result = self.client.send_async(request).await?;
        match result.status() {
            StatusCode::UNAUTHORIZED => Err(SpotifyApiError::InvalidToken),
            s if s.is_success() => Ok(()),
            s => Err(SpotifyApiError::BadStatus(s.as_u16())),
        }
    }
}

impl CachedSpotifyClient {
    async fn get_artist_no_cache(&self, id: &str) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let uri = self.uri(format!("/v1/artists/{}", id), None).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn get_artist_albums_no_cache(
        &self,
        id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let query = Self::make_query_params()
                .append_pair("include_groups", "album,single")
                .append_pair("country", "from_token")
                .append_pair("offset", &offset.to_string()[..])
                .append_pair("limit", &limit.to_string()[..])
                .finish();

            let uri = self
                .uri(format!("/v1/artists/{}/albums", id), Some(query))
                .unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn get_artist_top_tracks_no_cache(&self, id: &str) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let query = Self::make_query_params()
                .append_pair("market", "from_token")
                .finish();

            let uri = self
                .uri(format!("/v1/artists/{}/top-tracks", id), Some(query))
                .unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn is_album_saved(&self, id: &str) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let query = Self::make_query_params().append_pair("ids", id).finish();
            let uri = self
                .uri("/v1/me/albums/contains".to_string(), Some(query))
                .unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn raw_save_album(&self, id: &str) -> Result<(), SpotifyApiError> {
        self.send_req_no_response(|token| {
            let query = Self::make_query_params().append_pair("ids", id).finish();
            let uri = self.uri("/v1/me/albums".to_string(), Some(query)).unwrap();
            Request::put(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn raw_remove_saved_album(&self, id: &str) -> Result<(), SpotifyApiError> {
        self.send_req_no_response(|token| {
            let query = Self::make_query_params().append_pair("ids", id).finish();
            let uri = self.uri("/v1/me/albums".to_string(), Some(query)).unwrap();
            Request::delete(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn get_album_no_cache(&self, id: &str) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let uri = self.uri(format!("/v1/albums/{}", id), None).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn get_playlist_no_cache(&self, id: &str) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let uri = self.uri(format!("/v1/playlists/{}", id), None).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn get_saved_albums_no_cache(
        &self,
        offset: u32,
        limit: u32,
    ) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let query = Self::make_query_params()
                .append_pair("offset", &offset.to_string()[..])
                .append_pair("limit", &limit.to_string()[..])
                .finish();

            let uri = self.uri("/v1/me/albums".to_string(), Some(query)).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn get_saved_playlists_no_cache(
        &self,
        offset: u32,
        limit: u32,
    ) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let query = Self::make_query_params()
                .append_pair("offset", &offset.to_string()[..])
                .append_pair("limit", &limit.to_string()[..])
                .finish();

            let uri = self
                .uri("/v1/me/playlists".to_string(), Some(query))
                .unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    async fn search_no_cache(
        &self,
        query: String,
        offset: u32,
        limit: u32,
    ) -> Result<String, SpotifyApiError> {
        self.send_req(|token| {
            let query = SearchQuery {
                query,
                types: vec![SearchType::Album, SearchType::Artist],
                limit,
                offset,
            };

            let uri = self
                .uri("/v1/search".to_string(), Some(query.into_query_string()))
                .unwrap();

            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }
}

impl SpotifyApiClient for CachedSpotifyClient {
    fn update_token(&self, new_token: String) {
        if let Ok(mut token) = self.token.lock() {
            *token = Some(new_token)
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

    fn get_saved_playlists(
        &self,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<PlaylistDescription>>> {
        Box::pin(async move {
            let cache_request =
                self.cache_request(format!("net/me_playlists_{}_{}.json", offset, limit), None);

            let text = cache_request
                .or_else_try_write(
                    || self.get_saved_playlists_no_cache(offset, limit),
                    CacheExpiry::expire_in_seconds(3600),
                )
                .await?;

            let page = from_str::<Page<Playlist>>(&text)?;

            let albums = page
                .items
                .into_iter()
                .map(|playlist| playlist.into())
                .collect::<Vec<PlaylistDescription>>();

            Ok(albums)
        })
    }

    fn get_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let album = self
                .cache_request(format!("net/album_{}.json", id), None)
                .or_else_try_write(
                    || self.get_album_no_cache(&id[..]),
                    CacheExpiry::expire_in_hours(24),
                )
                .await?;
            let mut album: AlbumDescription = from_str::<Album>(&album)?.into();

            let liked = self
                .cache_request(format!("net/album_liked_{}.json", id), None)
                .or_else_try_write(
                    || self.is_album_saved(&id[..]),
                    CacheExpiry::expire_in_hours(2),
                )
                .await?;
            let liked = from_str::<Vec<bool>>(&liked)?;

            album.is_liked = liked[0];

            Ok(album)
        })
    }

    fn save_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired(&format!("net/album_liked_{}.json", id))
                .await
                .unwrap_or(());
            self.cache
                .set_expired_pattern("net", &*ME_ALBUMS_CACHE)
                .await
                .unwrap_or(());
            self.raw_save_album(&id[..]).await?;
            self.get_album(&id[..]).await
        })
    }

    fn remove_saved_album(&self, id: &str) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired(&format!("net/album_liked_{}.json", id))
                .await
                .unwrap_or(());
            self.cache
                .set_expired_pattern("net", &*ME_ALBUMS_CACHE)
                .await
                .unwrap_or(());
            self.raw_remove_saved_album(&id[..]).await
        })
    }

    fn get_playlist(&self, id: &str) -> BoxFuture<SpotifyResult<PlaylistDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let cache_request = self.cache_request(format!("net/playlist_{}.json", id), None);

            let text = cache_request
                .or_else_try_write(
                    || self.get_playlist_no_cache(&id[..]),
                    CacheExpiry::expire_in_hours(6),
                )
                .await?;

            Ok(from_str::<DetailedPlaylist>(&text)?.into())
        })
    }

    fn get_artist_albums(
        &self,
        id: &str,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>> {
        let id = id.to_owned();

        Box::pin(async move {
            let req = self.cache_request(
                format!("net/artist_albums_{}_{}_{}.json", id, offset, limit),
                None,
            );
            let albums = req.or_else_try_write(
                || self.get_artist_albums_no_cache(&id[..], offset, limit),
                CacheExpiry::expire_in_hours(24),
            );

            let albums: Page<Album> = from_str(&albums.await?)?;

            let albums = albums
                .items
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<AlbumDescription>>();

            Ok(albums)
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

            let albums = self.get_artist_albums(&id, 0, 20).await?;

            let req = self.cache_request(format!("net/artist_top_tracks_{}.json", id), None);
            let top_tracks = req.or_else_try_write(
                || self.get_artist_top_tracks_no_cache(&id[..]),
                CacheExpiry::expire_in_hours(24),
            );

            let artist: Artist = from_str(&artist.await?)?;
            let top_tracks: TopTracks = from_str(&top_tracks.await?)?;
            let top_tracks: Vec<SongDescription> = top_tracks.into();

            let result = ArtistDescription {
                name: artist.name,
                albums,
                top_tracks,
            };
            Ok(result)
        })
    }

    fn search(
        &self,
        query: &str,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<SearchResults>> {
        let query = query.to_owned();

        Box::pin(async move {
            let text = self.search_no_cache(query, offset, limit).await?;

            let results = from_str::<RawSearchResults>(&text)?;

            let albums = results
                .albums
                .unwrap_or_else(Page::empty)
                .items
                .into_iter()
                .map(|saved| saved.into())
                .collect::<Vec<AlbumDescription>>();

            let artists = results
                .artists
                .unwrap_or_else(Page::empty)
                .items
                .into_iter()
                .map(|saved| saved.into())
                .collect::<Vec<ArtistSummary>>();

            Ok(SearchResults { albums, artists })
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
