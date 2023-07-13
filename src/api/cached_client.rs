use futures::future::BoxFuture;
use futures::{join, FutureExt};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::from_slice;
use std::convert::Into;
use std::future::Future;

use super::cache::{CacheExpiry, CacheManager, CachePolicy, FetchResult};
use super::client::*;
use crate::app::models::*;

pub type SpotifyResult<T> = Result<T, SpotifyApiError>;

pub trait SpotifyApiClient {
    fn get_artist(&self, id: &str) -> BoxFuture<SpotifyResult<ArtistDescription>>;

    fn get_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumFullDescription>>;

    fn get_album_tracks(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<SongBatch>>;

    fn get_playlist(&self, id: &str) -> BoxFuture<SpotifyResult<PlaylistDescription>>;

    fn get_playlist_tracks(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<SongBatch>>;

    fn get_saved_albums(
        &self,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>>;

    fn get_saved_tracks(&self, offset: usize, limit: usize) -> BoxFuture<SpotifyResult<SongBatch>>;

    fn save_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>>;

    fn save_tracks(&self, ids: Vec<String>) -> BoxFuture<SpotifyResult<()>>;

    fn remove_saved_album(&self, id: &str) -> BoxFuture<SpotifyResult<()>>;

    fn remove_saved_tracks(&self, ids: Vec<String>) -> BoxFuture<SpotifyResult<()>>;

    fn get_saved_playlists(
        &self,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<PlaylistDescription>>>;

    fn add_to_playlist(&self, id: &str, uris: Vec<String>) -> BoxFuture<SpotifyResult<()>>;

    fn create_new_playlist(
        &self,
        name: &str,
        user_id: &str,
    ) -> BoxFuture<SpotifyResult<PlaylistDescription>>;

    fn remove_from_playlist(&self, id: &str, uris: Vec<String>) -> BoxFuture<SpotifyResult<()>>;

    fn update_playlist_details(&self, id: &str, name: String) -> BoxFuture<SpotifyResult<()>>;

    fn search(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<SearchResults>>;

    fn get_artist_albums(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>>;

    fn get_user(&self, id: &str) -> BoxFuture<SpotifyResult<UserDescription>>;

    fn get_user_playlists(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<PlaylistDescription>>>;

    fn list_available_devices(&self) -> BoxFuture<SpotifyResult<Vec<ConnectDevice>>>;

    fn get_player_queue(&self) -> BoxFuture<SpotifyResult<Vec<SongDescription>>>;

    fn update_token(&self, token: String);

    fn player_pause(&self, device_id: String) -> BoxFuture<SpotifyResult<()>>;

    fn player_resume(&self, device_id: String) -> BoxFuture<SpotifyResult<()>>;

    fn player_next(&self, device_id: String) -> BoxFuture<SpotifyResult<()>>;

    fn player_seek(&self, device_id: String, pos: usize) -> BoxFuture<SpotifyResult<()>>;

    fn player_repeat(&self, device_id: String, mode: RepeatMode) -> BoxFuture<SpotifyResult<()>>;

    fn player_shuffle(&self, device_id: String, shuffle: bool) -> BoxFuture<SpotifyResult<()>>;

    fn player_volume(&self, device_id: String, volume: u8) -> BoxFuture<SpotifyResult<()>>;

    fn player_play_in_context(
        &self,
        device_id: String,
        context: String,
        offset: usize,
    ) -> BoxFuture<SpotifyResult<()>>;

    fn player_play_no_context(
        &self,
        device_id: String,
        uris: Vec<String>,
        offset: usize,
    ) -> BoxFuture<SpotifyResult<()>>;

    fn player_state(&self) -> BoxFuture<SpotifyResult<ConnectPlayerState>>;

    fn get_followed_artists(
        &self, 
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Artists>>;
}

enum SpotCacheKey<'a> {
    SavedAlbums(usize, usize),
    SavedTracks(usize, usize),
    SavedPlaylists(usize, usize),
    Album(&'a str),
    AlbumLiked(&'a str),
    AlbumTracks(&'a str, usize, usize),
    Playlist(&'a str),
    PlaylistTracks(&'a str, usize, usize),
    ArtistAlbums(&'a str, usize, usize),
    Artist(&'a str),
    ArtistTopTracks(&'a str),
    User(&'a str),
    UserPlaylists(&'a str, usize, usize),
}

impl<'a> SpotCacheKey<'a> {
    fn into_raw(self) -> String {
        match self {
            Self::SavedAlbums(offset, limit) => format!("me_albums_{offset}_{limit}.json"),
            Self::SavedTracks(offset, limit) => format!("me_tracks_{offset}_{limit}.json"),
            Self::SavedPlaylists(offset, limit) => format!("me_playlists_{offset}_{limit}.json"),
            Self::Album(id) => format!("album_{id}.json"),
            Self::AlbumTracks(id, offset, limit) => {
                format!("album_item_{id}_{offset}_{limit}.json")
            }
            Self::AlbumLiked(id) => format!("album_liked_{id}.json"),
            Self::Playlist(id) => format!("playlist_{id}.json"),
            Self::PlaylistTracks(id, offset, limit) => {
                format!("playlist_item_{id}_{offset}_{limit}.json")
            }
            Self::ArtistAlbums(id, offset, limit) => {
                format!("artist_albums_{id}_{offset}_{limit}.json")
            }
            Self::Artist(id) => format!("artist_{id}.json"),
            Self::ArtistTopTracks(id) => format!("artist_top_tracks_{id}.json"),
            Self::User(id) => format!("user_{id}.json"),
            Self::UserPlaylists(id, offset, limit) => {
                format!("user_playlists_{id}_{offset}_{limit}.json")
            }
        }
    }
}

lazy_static! {
    pub static ref ME_TRACKS_CACHE: Regex = Regex::new(r"^me_tracks_\w+_\w+\.json$").unwrap();
    pub static ref ME_ALBUMS_CACHE: Regex = Regex::new(r"^me_albums_\w+_\w+\.json$").unwrap();
    pub static ref USER_CACHE: Regex =
        Regex::new(r"^me_(albums|playlists|tracks)_\w+_\w+\.json$").unwrap();
}

fn playlist_cache_key(id: &str) -> Regex {
    Regex::new(&format!(r"^playlist(_{id}|item_{id}_\w+_\w+)\.json$")).unwrap()
}

pub struct CachedSpotifyClient {
    client: SpotifyClient,
    cache: CacheManager,
}

impl CachedSpotifyClient {
    pub fn new() -> CachedSpotifyClient {
        CachedSpotifyClient {
            client: SpotifyClient::new(),
            cache: CacheManager::for_dir("spot/net").unwrap(),
        }
    }

    fn default_cache_policy(&self) -> CachePolicy {
        if self.client.has_token() {
            CachePolicy::Default
        } else {
            CachePolicy::IgnoreExpiry
        }
    }

    async fn wrap_write<T, O, F>(write: &F, etag: Option<String>) -> SpotifyResult<FetchResult>
    where
        O: Future<Output = SpotifyResult<SpotifyResponse<T>>>,
        F: Fn(Option<String>) -> O,
    {
        write(etag)
            .map(|r| {
                let SpotifyResponse {
                    kind,
                    max_age,
                    etag,
                } = r?;
                let expiry = CacheExpiry::expire_in_seconds(max_age, etag);
                SpotifyResult::Ok(match kind {
                    SpotifyResponseKind::Ok(content, _) => {
                        FetchResult::Modified(content.into_bytes(), expiry)
                    }
                    SpotifyResponseKind::NotModified => FetchResult::NotModified(expiry),
                })
            })
            .await
    }

    async fn cache_get_or_write<T, O, F>(
        &self,
        key: SpotCacheKey<'_>,
        cache_policy: Option<CachePolicy>,
        write: F,
    ) -> SpotifyResult<T>
    where
        O: Future<Output = SpotifyResult<SpotifyResponse<T>>>,
        F: Fn(Option<String>) -> O,
        T: DeserializeOwned,
    {
        let write = &write;
        let cache_key = key.into_raw();
        let raw = self
            .cache
            .get_or_write(
                &cache_key,
                cache_policy.unwrap_or_else(|| self.default_cache_policy()),
                |etag| Self::wrap_write(write, etag),
            )
            .await?;

        let result = from_slice::<T>(&raw);
        match result {
            Ok(t) => Ok(t),
            // parsing failed: cache is likely invalid, request again, ignoring cache
            Err(e) => {
                dbg!(&cache_key, e);
                let new_raw = self
                    .cache
                    .get_or_write(&cache_key, CachePolicy::IgnoreCached, |etag| {
                        Self::wrap_write(write, etag)
                    })
                    .await?;
                Ok(from_slice::<T>(&new_raw)?)
            }
        }
    }
}

impl SpotifyApiClient for CachedSpotifyClient {
    fn update_token(&self, new_token: String) {
        self.client.update_token(new_token)
    }

    fn get_saved_albums(
        &self,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>> {
        Box::pin(async move {
            let page = self
                .cache_get_or_write(SpotCacheKey::SavedAlbums(offset, limit), None, |etag| {
                    self.client
                        .get_saved_albums(offset, limit)
                        .etag(etag)
                        .send()
                })
                .await?;

            let albums = page
                .into_iter()
                .map(|saved| saved.album.into())
                .collect::<Vec<AlbumDescription>>();

            Ok(albums)
        })
    }

    fn get_saved_tracks(&self, offset: usize, limit: usize) -> BoxFuture<SpotifyResult<SongBatch>> {
        Box::pin(async move {
            let page = self
                .cache_get_or_write(SpotCacheKey::SavedTracks(offset, limit), None, |etag| {
                    self.client
                        .get_saved_tracks(offset, limit)
                        .etag(etag)
                        .send()
                })
                .await?;

            Ok(page.into())
        })
    }

    fn get_saved_playlists(
        &self,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<PlaylistDescription>>> {
        Box::pin(async move {
            let page = self
                .cache_get_or_write(SpotCacheKey::SavedPlaylists(offset, limit), None, |etag| {
                    self.client
                        .get_saved_playlists(offset, limit)
                        .etag(etag)
                        .send()
                })
                .await?;

            let albums = page
                .into_iter()
                .map(|playlist| playlist.into())
                .collect::<Vec<PlaylistDescription>>();

            Ok(albums)
        })
    }

    fn add_to_playlist(&self, id: &str, uris: Vec<String>) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired_pattern(&playlist_cache_key(&id))
                .await
                .unwrap_or(());

            self.client
                .add_to_playlist(&id, uris)
                .send_no_response()
                .await?;
            Ok(())
        })
    }

    fn create_new_playlist(
        &self,
        name: &str,
        user_id: &str,
    ) -> BoxFuture<SpotifyResult<PlaylistDescription>> {
        let name = name.to_owned();
        let user_id = user_id.to_owned();

        Box::pin(async move {
            let playlist = self
                .client
                .create_new_playlist(&name, &user_id)
                .send()
                .await?
                .deserialize()
                .unwrap();

            Ok(playlist.into())
        })
    }

    fn remove_from_playlist(&self, id: &str, uris: Vec<String>) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired_pattern(&playlist_cache_key(&id))
                .await
                .unwrap_or(());

            self.client
                .remove_from_playlist(&id, uris)
                .send_no_response()
                .await?;
            Ok(())
        })
    }

    fn update_playlist_details(&self, id: &str, name: String) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired_pattern(&playlist_cache_key(&id))
                .await
                .unwrap_or(());

            self.client
                .update_playlist_details(&id, name)
                .send_no_response()
                .await?;

            Ok(())
        })
    }

    fn get_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumFullDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let album = self.cache_get_or_write(SpotCacheKey::Album(&id), None, |etag| {
                self.client.get_album(&id).etag(etag).send()
            });

            let liked = self.cache_get_or_write(
                SpotCacheKey::AlbumLiked(&id),
                Some(if self.client.has_token() {
                    CachePolicy::Revalidate
                } else {
                    CachePolicy::IgnoreExpiry
                }),
                |etag| self.client.is_album_saved(&id).etag(etag).send(),
            );

            let (album, liked) = join!(album, liked);

            let mut album: AlbumFullDescription = album?.into();
            album.description.is_liked = liked?[0];

            Ok(album)
        })
    }

    fn save_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let _ = self.cache.set_expired_pattern(&ME_ALBUMS_CACHE).await;
            self.client.save_album(&id).send_no_response().await?;
            self.get_album(&id[..]).await.map(|a| a.description)
        })
    }

    fn save_tracks(&self, ids: Vec<String>) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(async move {
            let _ = self.cache.set_expired_pattern(&ME_TRACKS_CACHE).await;
            self.client.save_tracks(ids).send_no_response().await?;
            Ok(())
        })
    }

    fn remove_saved_album(&self, id: &str) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            let _ = self.cache.set_expired_pattern(&ME_ALBUMS_CACHE).await;
            self.client.remove_saved_album(&id).send_no_response().await
        })
    }

    fn remove_saved_tracks(&self, ids: Vec<String>) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(async move {
            let _ = self.cache.set_expired_pattern(&ME_TRACKS_CACHE).await;
            self.client
                .remove_saved_tracks(ids)
                .send_no_response()
                .await
        })
    }

    fn get_album_tracks(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<SongBatch>> {
        let id = id.to_owned();

        Box::pin(async move {
            let album = self.cache_get_or_write(
                SpotCacheKey::Album(&id),
                Some(CachePolicy::IgnoreExpiry),
                |etag| self.client.get_album(&id).etag(etag).send(),
            );

            let songs = self.cache_get_or_write(
                SpotCacheKey::AlbumTracks(&id, offset, limit),
                None,
                |etag| {
                    self.client
                        .get_album_tracks(&id, offset, limit)
                        .etag(etag)
                        .send()
                },
            );

            let (album, songs) = join!(album, songs);
            Ok((songs?, &album?.album).into())
        })
    }

    fn get_playlist(&self, id: &str) -> BoxFuture<SpotifyResult<PlaylistDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let playlist = self
                .cache_get_or_write(SpotCacheKey::Playlist(&id), None, |etag| {
                    self.client.get_playlist(&id).etag(etag).send()
                })
                .await?;

            Ok(playlist.into())
        })
    }

    fn get_playlist_tracks(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<SongBatch>> {
        let id = id.to_owned();

        Box::pin(async move {
            let songs = self
                .cache_get_or_write(
                    SpotCacheKey::PlaylistTracks(&id, offset, limit),
                    None,
                    |etag| {
                        self.client
                            .get_playlist_tracks(&id, offset, limit)
                            .etag(etag)
                            .send()
                    },
                )
                .await?;

            Ok(songs.into())
        })
    }

    fn get_artist_albums(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>> {
        let id = id.to_owned();

        Box::pin(async move {
            let albums = self
                .cache_get_or_write(
                    SpotCacheKey::ArtistAlbums(&id, offset, limit),
                    None,
                    |etag| {
                        self.client
                            .get_artist_albums(&id, offset, limit)
                            .etag(etag)
                            .send()
                    },
                )
                .await?;

            let albums = albums
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<AlbumDescription>>();

            Ok(albums)
        })
    }

    fn get_artist(&self, id: &str) -> BoxFuture<SpotifyResult<ArtistDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let artist = self.cache_get_or_write(SpotCacheKey::Artist(&id), None, |etag| {
                self.client.get_artist(&id).etag(etag).send()
            });

            let albums = self.get_artist_albums(&id, 0, 20);

            let top_tracks =
                self.cache_get_or_write(SpotCacheKey::ArtistTopTracks(&id), None, |etag| {
                    self.client.get_artist_top_tracks(&id).etag(etag).send()
                });

            let (artist, albums, top_tracks) = join!(artist, albums, top_tracks);

            let artist = artist?;
            let result = ArtistDescription {
                id: artist.id,
                name: artist.name,
                albums: albums?,
                top_tracks: top_tracks?.into(),
            };
            Ok(result)
        })
    }

    fn search(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<SearchResults>> {
        let query = query.to_owned();

        Box::pin(async move {
            let results = self
                .client
                .search(query, offset, limit)
                .send()
                .await?
                .deserialize()
                .ok_or(SpotifyApiError::NoContent)?;

            let albums = results
                .albums
                .unwrap_or_default()
                .into_iter()
                .map(|saved| saved.into())
                .collect::<Vec<AlbumDescription>>();

            let artists = results
                .artists
                .unwrap_or_default()
                .into_iter()
                .map(|saved| saved.into())
                .collect::<Vec<ArtistSummary>>();

            Ok(SearchResults { albums, artists })
        })
    }

    fn get_user_playlists(
        &self,
        id: &str,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<PlaylistDescription>>> {
        let id = id.to_owned();

        Box::pin(async move {
            let playlists = self
                .cache_get_or_write(
                    SpotCacheKey::UserPlaylists(&id, offset, limit),
                    None,
                    |etag| {
                        self.client
                            .get_user_playlists(&id, offset, limit)
                            .etag(etag)
                            .send()
                    },
                )
                .await?;

            let playlists = playlists
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<PlaylistDescription>>();

            Ok(playlists)
        })
    }

    fn get_user(&self, id: &str) -> BoxFuture<SpotifyResult<UserDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let user = self.cache_get_or_write(SpotCacheKey::User(&id), None, |etag| {
                self.client.get_user(&id).etag(etag).send()
            });

            let playlists = self.get_user_playlists(&id, 0, 30);

            let (user, playlists) = join!(user, playlists);

            let user = user?;
            let result = UserDescription {
                id: user.id,
                name: user.display_name,
                playlists: playlists?,
            };
            Ok(result)
        })
    }

    fn list_available_devices(&self) -> BoxFuture<SpotifyResult<Vec<ConnectDevice>>> {
        Box::pin(async move {
            let devices = self
                .client
                .get_player_devices()
                .send()
                .await?
                .deserialize()
                .ok_or(SpotifyApiError::NoContent)?;
            Ok(devices
                .devices
                .into_iter()
                .filter(|d| {
                    debug!("found device: {:?}", d);
                    !d.is_restricted
                })
                .map(ConnectDevice::from)
                .collect())
        })
    }

    fn get_player_queue(&self) -> BoxFuture<SpotifyResult<Vec<SongDescription>>> {
        Box::pin(async move {
            let queue = self
                .client
                .get_player_queue()
                .send()
                .await?
                .deserialize()
                .ok_or(SpotifyApiError::NoContent)?;
            Ok(queue.into())
        })
    }

    fn player_pause(&self, device_id: String) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(self.client.player_pause(&device_id).send_no_response())
    }

    fn player_resume(&self, device_id: String) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(self.client.player_resume(&device_id).send_no_response())
    }

    fn player_play_in_context(
        &self,
        device_id: String,
        context_uri: String,
        offset: usize,
    ) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(
            self.client
                .player_set_playing(
                    &device_id,
                    PlayRequest::Contextual {
                        context_uri,
                        offset: PlayOffset {
                            position: offset as u32,
                        },
                    },
                )
                .send_no_response(),
        )
    }

    fn player_play_no_context(
        &self,
        device_id: String,
        uris: Vec<String>,
        offset: usize,
    ) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(
            self.client
                .player_set_playing(
                    &device_id,
                    PlayRequest::Uris {
                        uris,
                        offset: PlayOffset {
                            position: offset as u32,
                        },
                    },
                )
                .send_no_response(),
        )
    }

    fn player_next(&self, device_id: String) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(self.client.player_next(&device_id).send_no_response())
    }

    fn player_seek(&self, device_id: String, pos: usize) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(self.client.player_seek(&device_id, pos).send_no_response())
    }

    fn player_state(&self) -> BoxFuture<SpotifyResult<ConnectPlayerState>> {
        Box::pin(async move {
            let result = self
                .client
                .player_state()
                .send()
                .await?
                .deserialize()
                .ok_or(SpotifyApiError::NoContent)?;
            Ok(result.into())
        })
    }

    fn player_repeat(&self, device_id: String, mode: RepeatMode) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(
            self.client
                .player_repeat(
                    &device_id,
                    match mode {
                        RepeatMode::Song => "track",
                        RepeatMode::Playlist => "context",
                        RepeatMode::None => "off",
                    },
                )
                .send_no_response(),
        )
    }

    fn player_shuffle(&self, device_id: String, shuffle: bool) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(
            self.client
                .player_shuffle(&device_id, shuffle)
                .send_no_response(),
        )
    }

    fn player_volume(&self, device_id: String, volume: u8) -> BoxFuture<SpotifyResult<()>> {
        Box::pin(
            self.client
                .player_volume(&device_id, volume)
                .send_no_response(),
        )
    }

    fn get_followed_artists(
            &self, 
            offset: usize,
            limit: usize,
        ) -> BoxFuture<SpotifyResult<Artists>> {
            Box::pin(async move {
                let result = self
                    .client
                    .get_followed_artists(offset, limit)
                    .send()
                    .await?
                    .deserialize()
                    .ok_or(SpotifyApiError::NoContent)?;
                Ok(result.into())
            })
    }
}

#[cfg(test)]
pub mod tests {

    use crate::api::api_models::*;

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
