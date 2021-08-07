use futures::future::BoxFuture;
use futures::{join, FutureExt};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::from_slice;
use std::convert::Into;
use std::future::Future;

use super::cache::{CacheExpiry, CacheManager, CachePolicy, FetchResult};
use super::client::{SpotifyApiError, SpotifyClient, SpotifyResponse, SpotifyResponseKind};
use crate::app::models::*;

lazy_static! {
    pub static ref ME_ALBUMS_CACHE: Regex =
        Regex::new(r"^me_albums_\w+_\w+\.json\.expiry$").unwrap();
    pub static ref USER_CACHE: Regex =
        Regex::new(r"^me_(albums|playlists)_\w+_\w+\.json(\.expiry)?$").unwrap();
    pub static ref ALL_CACHE: Regex =
        Regex::new(r"^(me_albums_|me_playlists_|album_|playlist_|artist_)\w+\.json(\.expiry)?$")
            .unwrap();
}

fn _playlist_cache(id: &str) -> Regex {
    Regex::new(&format!(
        r"^playlist(_{}|item_{}_\w+_\w+)\.json\.expiry$",
        id, id
    ))
    .unwrap()
}

pub type SpotifyResult<T> = Result<T, SpotifyApiError>;

pub trait SpotifyApiClient {
    fn get_artist(&self, id: &str) -> BoxFuture<SpotifyResult<ArtistDescription>>;

    fn get_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>>;

    fn get_album_info(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDetailedInfo>>;

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

    fn save_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>>;

    fn remove_saved_album(&self, id: &str) -> BoxFuture<SpotifyResult<()>>;

    fn get_saved_playlists(
        &self,
        offset: usize,
        limit: usize,
    ) -> BoxFuture<SpotifyResult<Vec<PlaylistDescription>>>;

    fn add_to_playlist(&self, id: &str, uris: Vec<String>) -> BoxFuture<SpotifyResult<()>>;

    fn remove_from_playlist(&self, id: &str, uris: Vec<String>) -> BoxFuture<SpotifyResult<()>>;

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

    fn update_token(&self, token: String);
}

enum SpotCacheKey<'a> {
    SavedAlbums(usize, usize),
    SavedPlaylists(usize, usize),
    Album(&'a str),
    AlbumLiked(&'a str),
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
            Self::SavedAlbums(offset, limit) => format!("me_albums_{}_{}.json", offset, limit),
            Self::SavedPlaylists(offset, limit) => {
                format!("me_playlists_{}_{}.json", offset, limit)
            }
            Self::Album(id) => format!("album_{}.json", id),
            Self::AlbumLiked(id) => format!("album_liked_{}.json", id),
            Self::Playlist(id) => format!("playlist_{}.json", id),
            Self::PlaylistTracks(id, offset, limit) => {
                format!("playlist_item_{}_{}_{}.json", id, offset, limit)
            }
            Self::ArtistAlbums(id, offset, limit) => {
                format!("artist_albums_{}_{}_{}.json", id, offset, limit)
            }
            Self::Artist(id) => format!("artist_{}.json", id),
            Self::ArtistTopTracks(id) => format!("artist_top_tracks_{}.json", id),
            Self::User(id) => format!("user_{}.json", id),
            Self::UserPlaylists(id, offset, limit) => {
                format!("user_playlists_{}_{}_{}.json", id, offset, limit)
            }
        }
    }
}

pub struct CachedSpotifyClient {
    client: SpotifyClient,
    cache: CacheManager,
}

impl CachedSpotifyClient {
    pub fn new() -> CachedSpotifyClient {
        CachedSpotifyClient {
            client: SpotifyClient::new(),
            cache: CacheManager::new(&["spot/net"]).unwrap(),
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
                let expiry = CacheExpiry::expire_in_seconds(u64::max(max_age, 10), etag);
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
        let cache_key = format!("spot/net/{}", key.into_raw());
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
                dbg!(e);
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
                .map(|playlist| playlist.into_playlist_description(limit, offset))
                .collect::<Vec<PlaylistDescription>>();

            Ok(albums)
        })
    }

    fn add_to_playlist(&self, id: &str, uris: Vec<String>) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired_pattern("spot/net", &_playlist_cache(&id))
                .await
                .unwrap_or(());

            self.client
                .add_to_playlist(&id, uris)
                .send_no_response()
                .await?;
            Ok(())
        })
    }

    fn remove_from_playlist(&self, id: &str, uris: Vec<String>) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired_pattern("spot/net", &_playlist_cache(&id))
                .await
                .unwrap_or(());

            self.client
                .remove_from_playlist(&id, uris)
                .send_no_response()
                .await?;
            Ok(())
        })
    }

    fn get_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>> {
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

            let mut album: AlbumDescription = album?.into();
            album.is_liked = liked?[0];

            Ok(album)
        })
    }

    fn get_album_info(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDetailedInfo>> {
        let id = id.to_owned();

        Box::pin(async move {
            let album_info = self.cache_get_or_write(SpotCacheKey::Album(&id), None, |etag| {
                self.client.get_album_info(&id).etag(etag).send()
            });

            let album = self.get_album(&id);

            let (album_info, album) = join!(album_info, album);

            let album_detail = AlbumDetailedInfo::new(album_info?, album?);
            Ok(album_detail)
        })
    }

    fn save_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired_pattern("spot/net", &*ME_ALBUMS_CACHE)
                .await
                .unwrap_or(());
            self.client.save_album(&id).send_no_response().await?;
            self.get_album(&id[..]).await
        })
    }

    fn remove_saved_album(&self, id: &str) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired_pattern("spot/net", &*ME_ALBUMS_CACHE)
                .await
                .unwrap_or(());
            self.client.remove_saved_album(&id).send_no_response().await
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

            Ok(playlist.into_playlist_description(100, 0))
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

            let batch = Batch {
                batch_size: limit,
                offset,
                total: songs.total,
            };
            let songs: Vec<SongDescription> = songs.into();
            Ok(SongBatch { songs, batch })
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

    fn get_artist(&self, id: &str) -> BoxFuture<Result<ArtistDescription, SpotifyApiError>> {
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
                .map(|a| a.into_playlist_description(limit, offset))
                .collect::<Vec<PlaylistDescription>>();

            Ok(playlists)
        })
    }

    fn get_user(&self, id: &str) -> BoxFuture<Result<UserDescription, SpotifyApiError>> {
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
