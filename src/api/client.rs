use form_urlencoded::Serializer;
use isahc::config::Configurable;
use isahc::http::{method::Method, request::Builder, StatusCode, Uri};
use isahc::{AsyncReadResponseExt, HttpClient, Request};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{de::Deserialize, Serialize};
use serde_json::from_str;
use std::convert::Into;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Mutex;
use thiserror::Error;

pub use super::api_models::*;
use super::cache::CacheError;

const SPOTIFY_HOST: &str = "api.spotify.com";

// https://url.spec.whatwg.org/#path-percent-encode-set
const PATH_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

fn make_query_params<'a>() -> Serializer<'a, String> {
    Serializer::new(String::new())
}

pub(crate) struct SpotifyRequest<'a, Body, Response> {
    client: &'a SpotifyClient,
    request: Builder,
    body: Body,
    _type: PhantomData<Response>,
}

impl<'a, B, R> SpotifyRequest<'a, B, R>
where
    B: Into<isahc::AsyncBody>,
{
    fn method(mut self, method: Method) -> Self {
        self.request = self.request.method(method);
        self
    }

    fn uri(mut self, path: String, query: Option<&str>) -> Self {
        let path_and_query = match query {
            None => path,
            Some(query) => format!("{path}?{query}"),
        };
        let uri = Uri::builder()
            .scheme("https")
            .authority(SPOTIFY_HOST)
            .path_and_query(&path_and_query[..])
            .build()
            .unwrap();
        self.request = self.request.uri(uri);
        self
    }

    fn authenticated(mut self) -> Result<Self, SpotifyApiError> {
        let token = self.client.token.lock().unwrap();
        let token = token.as_ref().ok_or(SpotifyApiError::NoToken)?;
        self.request = self
            .request
            .header("Authorization", format!("Bearer {token}"));
        Ok(self)
    }

    pub(crate) fn etag(mut self, etag: Option<String>) -> Self {
        if let Some(etag) = etag {
            self.request = self.request.header("If-None-Match", etag);
        }
        self
    }

    pub(crate) fn json_body<NewBody>(self, body: NewBody) -> SpotifyRequest<'a, Vec<u8>, R>
    where
        NewBody: Serialize,
    {
        let Self {
            client,
            request,
            _type,
            ..
        } = self;
        SpotifyRequest {
            client,
            request: request.header("Content-Type", "application/json"),
            body: serde_json::to_vec(&body).unwrap(),
            _type,
        }
    }

    pub(crate) async fn send(self) -> Result<SpotifyResponse<R>, SpotifyApiError> {
        let Self {
            client,
            request,
            body,
            ..
        } = self.authenticated()?;
        client.send_req(request.body(body).unwrap()).await
    }

    pub(crate) async fn send_no_response(self) -> Result<(), SpotifyApiError> {
        let Self {
            client,
            request,
            body,
            ..
        } = self.authenticated()?;
        client
            .send_req_no_response(request.body(body).unwrap())
            .await
    }
}

pub(crate) enum SpotifyResponseKind<T> {
    Ok(String, PhantomData<T>),
    NotModified,
}

pub(crate) struct SpotifyResponse<T> {
    pub kind: SpotifyResponseKind<T>,
    pub max_age: u64,
    pub etag: Option<String>,
}

impl<'a, T> SpotifyResponse<T>
where
    T: Deserialize<'a>,
{
    pub(crate) fn deserialize(&'a self) -> Option<T> {
        if let SpotifyResponseKind::Ok(ref content, _) = self.kind {
            from_str(content)
                .map_err(|e| error!("Deserialization failed: {}", e))
                .ok()
        } else {
            None
        }
    }
}

#[derive(Error, Debug)]
pub enum SpotifyApiError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("No token")]
    NoToken,
    #[error("No content from request")]
    NoContent,
    #[error("Request rate exceeded")]
    TooManyRequests,
    #[error("Request failed ({0}): {1}")]
    BadStatus(u16, String),
    #[error(transparent)]
    ClientError(#[from] isahc::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    CacheError(#[from] CacheError),
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),
    #[error(transparent)]
    ConversionError(#[from] std::string::FromUtf8Error),
}

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

    pub(crate) fn request<T>(&self) -> SpotifyRequest<'_, (), T> {
        SpotifyRequest {
            client: self,
            request: Builder::new(),
            body: (),
            _type: PhantomData,
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

    fn clear_token(&self) {
        if let Ok(mut token) = self.token.lock() {
            *token = None
        }
    }

    fn parse_cache_control(cache_control: &str) -> Option<u64> {
        cache_control
            .split(',')
            .find(|s| s.trim().starts_with("max-age="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| u64::from_str(s).ok())
    }

    async fn send_req<B, T>(
        &self,
        request: Request<B>,
    ) -> Result<SpotifyResponse<T>, SpotifyApiError>
    where
        B: Into<isahc::AsyncBody>,
    {
        let mut result = self.client.send_async(request).await?;

        let etag = result
            .headers()
            .get("etag")
            .and_then(|header| header.to_str().ok())
            .map(|s| s.to_owned());

        let cache_control = result
            .headers()
            .get("cache-control")
            .and_then(|header| header.to_str().ok())
            .and_then(Self::parse_cache_control);

        match result.status() {
            StatusCode::NO_CONTENT => Err(SpotifyApiError::NoContent),
            s if s.is_success() => Ok(SpotifyResponse {
                kind: SpotifyResponseKind::Ok(result.text().await?, PhantomData),
                max_age: cache_control.unwrap_or(10),
                etag,
            }),
            StatusCode::UNAUTHORIZED => {
                self.clear_token();
                Err(SpotifyApiError::InvalidToken)
            }
            StatusCode::TOO_MANY_REQUESTS => Err(SpotifyApiError::TooManyRequests),
            StatusCode::NOT_MODIFIED => Ok(SpotifyResponse {
                kind: SpotifyResponseKind::NotModified,
                max_age: cache_control.unwrap_or(10),
                etag,
            }),
            s => Err(SpotifyApiError::BadStatus(
                s.as_u16(),
                result
                    .text()
                    .await
                    .unwrap_or_else(|_| "(no details available)".to_string()),
            )),
        }
    }

    async fn send_req_no_response<B>(&self, request: Request<B>) -> Result<(), SpotifyApiError>
    where
        B: Into<isahc::AsyncBody>,
    {
        let mut result = self.client.send_async(request).await?;
        match result.status() {
            StatusCode::UNAUTHORIZED => {
                self.clear_token();
                Err(SpotifyApiError::InvalidToken)
            }
            StatusCode::TOO_MANY_REQUESTS => Err(SpotifyApiError::TooManyRequests),
            StatusCode::NOT_MODIFIED => Ok(()),
            s if s.is_success() => Ok(()),
            s => Err(SpotifyApiError::BadStatus(
                s.as_u16(),
                result
                    .text()
                    .await
                    .unwrap_or_else(|_| "(no details available)".to_string()),
            )),
        }
    }
}

impl SpotifyClient {
    pub(crate) fn get_artist(&self, id: &str) -> SpotifyRequest<'_, (), Artist> {
        self.request()
            .method(Method::GET)
            .uri(format!("/v1/artists/{id}"), None)
    }

    pub(crate) fn get_artist_albums(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> SpotifyRequest<'_, (), Page<Album>> {
        let query = make_query_params()
            .append_pair("include_groups", "album,single")
            .append_pair("country", "from_token")
            .append_pair("offset", &offset.to_string()[..])
            .append_pair("limit", &limit.to_string()[..])
            .finish();

        self.request()
            .method(Method::GET)
            .uri(format!("/v1/artists/{id}/albums"), Some(&query))
    }

    pub(crate) fn get_artist_top_tracks(&self, id: &str) -> SpotifyRequest<'_, (), TopTracks> {
        let query = make_query_params()
            .append_pair("market", "from_token")
            .finish();

        self.request()
            .method(Method::GET)
            .uri(format!("/v1/artists/{id}/top-tracks"), Some(&query))
    }

    pub(crate) fn is_album_saved(&self, id: &str) -> SpotifyRequest<'_, (), Vec<bool>> {
        let query = make_query_params().append_pair("ids", id).finish();
        self.request()
            .method(Method::GET)
            .uri("/v1/me/albums/contains".to_string(), Some(&query))
    }

    pub(crate) fn save_album(&self, id: &str) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params().append_pair("ids", id).finish();
        self.request()
            .method(Method::PUT)
            .uri("/v1/me/albums".to_string(), Some(&query))
    }

    pub(crate) fn save_tracks(&self, ids: Vec<String>) -> SpotifyRequest<'_, Vec<u8>, ()> {
        self.request()
            .method(Method::PUT)
            .uri("/v1/me/tracks".to_string(), None)
            .json_body(Ids { ids })
    }

    pub(crate) fn remove_saved_album(&self, id: &str) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params().append_pair("ids", id).finish();
        self.request()
            .method(Method::DELETE)
            .uri("/v1/me/albums".to_string(), Some(&query))
    }

    pub(crate) fn remove_saved_tracks(&self, ids: Vec<String>) -> SpotifyRequest<'_, Vec<u8>, ()> {
        self.request()
            .method(Method::DELETE)
            .uri("/v1/me/tracks".to_string(), None)
            .json_body(Ids { ids })
    }

    pub(crate) fn get_album(&self, id: &str) -> SpotifyRequest<'_, (), FullAlbum> {
        self.request()
            .method(Method::GET)
            .uri(format!("/v1/albums/{id}"), None)
    }

    pub(crate) fn get_album_tracks(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> SpotifyRequest<'_, (), Page<AlbumTrackItem>> {
        let query = make_query_params()
            .append_pair("offset", &offset.to_string()[..])
            .append_pair("limit", &limit.to_string()[..])
            .finish();

        self.request()
            .method(Method::GET)
            .uri(format!("/v1/albums/{id}/tracks"), Some(&query))
    }

    pub(crate) fn get_playlist(&self, id: &str) -> SpotifyRequest<'_, (), Playlist> {
        let query = make_query_params()
            .append_pair("market", "from_token")
            // why still grab the tracks field? 
            // the model still expects the appearance of a tracks field
            .append_pair(
                "fields",
                "id,name,images,owner,tracks(total)",
            )
            .finish();
        self.request()
            .method(Method::GET)
            .uri(format!("/v1/playlists/{id}"), Some(&query))
    }

    pub(crate) fn get_playlist_tracks(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> SpotifyRequest<'_, (), Page<PlaylistTrack>> {
        let query = make_query_params()
            .append_pair("market", "from_token")
            .append_pair("offset", &offset.to_string()[..])
            .append_pair("limit", &limit.to_string()[..])
            .finish();

        self.request()
            .method(Method::GET)
            .uri(format!("/v1/playlists/{id}/tracks"), Some(&query))
    }

    pub(crate) fn add_to_playlist(
        &self,
        playlist: &str,
        uris: Vec<String>,
    ) -> SpotifyRequest<'_, Vec<u8>, ()> {
        self.request()
            .method(Method::POST)
            .uri(format!("/v1/playlists/{playlist}/tracks"), None)
            .json_body(Uris { uris })
    }

    pub(crate) fn create_new_playlist(
        &self,
        name: &str,
        user_id: &str,
    ) -> SpotifyRequest<'_, Vec<u8>, Playlist> {
        self.request()
            .method(Method::POST)
            .uri(format!("/v1/users/{user_id}/playlists"), None)
            .json_body(Name { name })
    }

    pub(crate) fn remove_from_playlist(
        &self,
        playlist: &str,
        uris: Vec<String>,
    ) -> SpotifyRequest<'_, Vec<u8>, ()> {
        self.request()
            .method(Method::DELETE)
            .uri(format!("/v1/playlists/{playlist}/tracks"), None)
            .json_body(Uris { uris })
    }

    pub(crate) fn update_playlist_details(
        &self,
        playlist: &str,
        name: String,
    ) -> SpotifyRequest<'_, Vec<u8>, ()> {
        self.request()
            .method(Method::PUT)
            .uri(format!("/v1/playlists/{playlist}"), None)
            .json_body(PlaylistDetails { name })
    }

    pub(crate) fn get_saved_albums(
        &self,
        offset: usize,
        limit: usize,
    ) -> SpotifyRequest<'_, (), Page<SavedAlbum>> {
        let query = make_query_params()
            .append_pair("offset", &offset.to_string()[..])
            .append_pair("limit", &limit.to_string()[..])
            .finish();

        self.request()
            .method(Method::GET)
            .uri("/v1/me/albums".to_string(), Some(&query))
    }

    pub(crate) fn get_saved_tracks(
        &self,
        offset: usize,
        limit: usize,
    ) -> SpotifyRequest<'_, (), Page<SavedTrack>> {
        let query = make_query_params()
            .append_pair("offset", &offset.to_string()[..])
            .append_pair("limit", &limit.to_string()[..])
            .finish();

        self.request()
            .method(Method::GET)
            .uri("/v1/me/tracks".to_string(), Some(&query))
    }

    pub(crate) fn get_saved_playlists(
        &self,
        offset: usize,
        limit: usize,
    ) -> SpotifyRequest<'_, (), Page<Playlist>> {
        let query = make_query_params()
            .append_pair("offset", &offset.to_string()[..])
            .append_pair("limit", &limit.to_string()[..])
            .finish();

        self.request()
            .method(Method::GET)
            .uri("/v1/me/playlists".to_string(), Some(&query))
    }

    pub(crate) fn search(
        &self,
        query: String,
        offset: usize,
        limit: usize,
    ) -> SpotifyRequest<'_, (), RawSearchResults> {
        let query = SearchQuery {
            query,
            types: vec![SearchType::Album, SearchType::Artist],
            limit,
            offset,
        };

        self.request()
            .method(Method::GET)
            .uri("/v1/search".to_string(), Some(&query.into_query_string()))
    }

    pub(crate) fn get_user(&self, id: &str) -> SpotifyRequest<'_, (), User> {
        let id = utf8_percent_encode(id, PATH_ENCODE_SET);
        self.request()
            .method(Method::GET)
            .uri(format!("/v1/users/{id}"), None)
    }

    pub(crate) fn get_user_playlists(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> SpotifyRequest<'_, (), Page<Playlist>> {
        let id = utf8_percent_encode(id, PATH_ENCODE_SET);
        let query = make_query_params()
            .append_pair("offset", &offset.to_string()[..])
            .append_pair("limit", &limit.to_string()[..])
            .finish();

        self.request()
            .method(Method::GET)
            .uri(format!("/v1/users/{id}/playlists"), Some(&query))
    }

    pub(crate) fn get_player_devices(&self) -> SpotifyRequest<'_, (), Devices> {
        self.request()
            .method(Method::GET)
            .uri("/v1/me/player/devices".to_string(), None)
    }

    pub(crate) fn get_player_queue(&self) -> SpotifyRequest<'_, (), PlayerQueue> {
        self.request()
            .method(Method::GET)
            .uri("/v1/me/player/queue".to_string(), None)
    }

    pub(crate) fn player_state(&self) -> SpotifyRequest<'_, (), PlayerState> {
        self.request()
            .method(Method::GET)
            .uri("/v1/me/player".to_string(), None)
    }

    pub(crate) fn player_resume(&self, device_id: &str) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params()
            .append_pair("device_id", device_id)
            .finish();
        self.request()
            .method(Method::PUT)
            .uri("/v1/me/player/play".to_string(), Some(&query))
    }

    pub(crate) fn player_set_playing(
        &self,
        device_id: &str,
        request: PlayRequest,
    ) -> SpotifyRequest<'_, Vec<u8>, ()> {
        let query = make_query_params()
            .append_pair("device_id", device_id)
            .finish();
        self.request()
            .method(Method::PUT)
            .uri("/v1/me/player/play".to_string(), Some(&query))
            .json_body(request)
    }

    pub(crate) fn player_pause(&self, device_id: &str) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params()
            .append_pair("device_id", device_id)
            .finish();
        self.request()
            .method(Method::PUT)
            .uri("/v1/me/player/pause".to_string(), Some(&query))
    }

    pub(crate) fn player_next(&self, device_id: &str) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params()
            .append_pair("device_id", device_id)
            .finish();
        self.request()
            .method(Method::PUT)
            .uri("/v1/me/player/next".to_string(), Some(&query))
    }

    pub(crate) fn player_seek(&self, device_id: &str, pos: usize) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params()
            .append_pair("device_id", device_id)
            .append_pair("position_ms", &pos.to_string()[..])
            .finish();

        self.request()
            .method(Method::PUT)
            .uri("/v1/me/player/seek".to_string(), Some(&query))
    }

    pub(crate) fn player_repeat(&self, device_id: &str, state: &str) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params()
            .append_pair("device_id", device_id)
            .append_pair("state", state)
            .finish();

        self.request()
            .method(Method::PUT)
            .uri("/v1/me/player/repeat".to_string(), Some(&query))
    }

    pub(crate) fn player_shuffle(
        &self,
        device_id: &str,
        shuffle: bool,
    ) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params()
            .append_pair("device_id", device_id)
            .append_pair("state", if shuffle { "true" } else { "false" })
            .finish();

        self.request()
            .method(Method::PUT)
            .uri("/v1/me/player/shuffle".to_string(), Some(&query))
    }

    pub(crate) fn player_volume(&self, device_id: &str, volume: u8) -> SpotifyRequest<'_, (), ()> {
        let query = make_query_params()
            .append_pair("device_id", device_id)
            .append_pair("volume_percent", &volume.to_string())
            .finish();

        self.request()
            .method(Method::PUT)
            .uri("/v1/me/player/volume".to_string(), Some(&query))
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_username_encoding() {
        let username = "anna.lafuente❤";
        let client = SpotifyClient::new();
        let req = client.get_user(username);
        assert_eq!(
            req.request
                .uri_ref()
                .and_then(|u| u.path_and_query())
                .unwrap()
                .as_str(),
            "/v1/users/anna.lafuente%E2%9D%A4"
        );
    }

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
