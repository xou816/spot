use form_urlencoded::Serializer;
use isahc::config::Configurable;
use isahc::http::{StatusCode, Uri};
use isahc::{AsyncReadResponseExt, HttpClient, Request};
use serde::Deserialize;
use serde_json::from_str;
use std::convert::Into;
use std::marker::PhantomData;
use std::sync::Mutex;
use thiserror::Error;

pub use super::api_models::*;

const SPOTIFY_HOST: &str = "api.spotify.com";

pub(crate) struct SpotifyResponse<T> {
    pub content: String,
    _type: PhantomData<T>,
}

impl<T> SpotifyResponse<T> {
    pub(crate) fn new(content: String) -> Self {
        Self {
            content,
            _type: PhantomData,
        }
    }
}

impl<'a, T> SpotifyResponse<T>
where
    T: Deserialize<'a>,
{
    pub(crate) fn deserialize(&'a self) -> serde_json::Result<T> {
        from_str(&self.content)
    }
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

pub(crate) type SpotifyRawResult<T> = Result<SpotifyResponse<T>, SpotifyApiError>;

pub(crate) struct SpotifyClient {
    token: Mutex<Option<String>>,
    client: HttpClient,
}

impl SpotifyClient {
    pub(crate) fn new() -> Self {
        let mut builder = HttpClient::builder();
        if cfg!(debug_assertions) {
            builder = builder.ssl_options(isahc::config::SslOption::DANGER_ACCEPT_INVALID_CERTS);
        }
        let client = builder.build().unwrap();
        Self {
            token: Mutex::new(None),
            client,
        }
    }

    pub(crate) fn has_token(&self) -> bool {
        self.token.lock().unwrap().is_some()
    }

    pub(crate) fn update_token(&self, new_token: String) {
        if let Ok(mut token) = self.token.lock() {
            *token = Some(new_token)
        }
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

    async fn send_req<B, F, T>(
        &self,
        make_request: F,
    ) -> Result<SpotifyResponse<T>, SpotifyApiError>
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
            s if s.is_success() => Ok(SpotifyResponse::new(result.text().await?)),
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

impl SpotifyClient {
    pub(crate) async fn get_artist(&self, id: &str) -> SpotifyRawResult<Artist> {
        self.send_req(|token| {
            let uri = self.uri(format!("/v1/artists/{}", id), None).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(())
                .unwrap()
        })
        .await
    }

    pub(crate) async fn get_artist_albums(
        &self,
        id: &str,
        offset: u32,
        limit: u32,
    ) -> SpotifyRawResult<Page<Album>> {
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

    pub(crate) async fn get_artist_top_tracks(&self, id: &str) -> SpotifyRawResult<TopTracks> {
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

    pub(crate) async fn is_album_saved(&self, id: &str) -> SpotifyRawResult<Vec<bool>> {
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

    pub(crate) async fn save_album(&self, id: &str) -> Result<(), SpotifyApiError> {
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

    pub(crate) async fn remove_saved_album(&self, id: &str) -> Result<(), SpotifyApiError> {
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

    pub(crate) async fn get_album(&self, id: &str) -> SpotifyRawResult<Album> {
        self.send_req(|token| {
            let uri = self.uri(format!("/v1/albums/{}", id), None).unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    pub(crate) async fn get_playlist(&self, id: &str) -> SpotifyRawResult<Playlist> {
        self.send_req(|token| {
            let query = Self::make_query_params()
                .append_pair("fields", "id,name,images,owner")
                .finish();
            let uri = self
                .uri(format!("/v1/playlists/{}", id), Some(query))
                .unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    pub(crate) async fn get_playlist_tracks(
        &self,
        id: &str,
        offset: u32,
        limit: u32,
    ) -> SpotifyRawResult<Page<PlaylistTrack>> {
        self.send_req(|token| {
            let query = Self::make_query_params()
                .append_pair("offset", &offset.to_string()[..])
                .append_pair("limit", &limit.to_string()[..])
                .finish();

            let uri = self
                .uri(format!("/v1/playlists/{}/tracks", id), Some(query))
                .unwrap();
            Request::get(uri)
                .header("Authorization", format!("Bearer {}", &token))
                .body(())
                .unwrap()
        })
        .await
    }

    pub(crate) async fn get_saved_albums(
        &self,
        offset: u32,
        limit: u32,
    ) -> SpotifyRawResult<Page<SavedAlbum>> {
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

    pub(crate) async fn get_saved_playlists(
        &self,
        offset: u32,
        limit: u32,
    ) -> SpotifyRawResult<Page<Playlist>> {
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

    pub(crate) async fn search(
        &self,
        query: String,
        offset: u32,
        limit: u32,
    ) -> SpotifyRawResult<RawSearchResults> {
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
