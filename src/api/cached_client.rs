use futures::future::BoxFuture;
use futures::FutureExt;
use regex::Regex;
use std::convert::Into;
use std::future::Future;

use super::api_models::*;
use super::cache::{CacheExpiry, CacheManager, CachePolicy, CacheRequest};
use super::client::{SpotifyApiError, SpotifyClient, SpotifyRawResult, SpotifyResponse};
use crate::app::models::*;

lazy_static! {
    static ref ME_ALBUMS_CACHE: Regex = Regex::new(r"^me_albums_\w+_\w+\.json\.expiry$").unwrap();
}

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

enum SpotCacheKey<'a> {
    SavedAlbums(u32, u32),
    SavedPlaylists(u32, u32),
    Album(&'a str),
    AlbumLiked(&'a str),
    Playlist(&'a str),
    PlaylistTracks(&'a str, u32, u32),
    ArtistAlbums(&'a str, u32, u32),
    Artist(&'a str),
    ArtistTopTracks(&'a str),
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

    async fn cache_get_or_write<'a, T, O, F>(
        &self,
        key: SpotCacheKey<'a>,
        write: F,
        expiry: CacheExpiry,
    ) -> SpotifyRawResult<T>
    where
        O: Future<Output = SpotifyRawResult<T>>,
        F: FnOnce() -> O,
    {

        let req = CacheRequest::for_resource(
            &self.cache,
            format!("spot/net/{}", key.into_raw()),
            self.default_cache_policy(),
        );

        let raw = req
            .get_or_write(
                move || write().map(|r| SpotifyResult::Ok(r?.content)),
                expiry,
            )
            .await?;

        Ok(SpotifyResponse::new(raw))
    }
}

impl SpotifyApiClient for CachedSpotifyClient {
    fn update_token(&self, new_token: String) {
        self.client.update_token(new_token)
    }

    fn get_saved_albums(
        &self,
        offset: u32,
        limit: u32,
    ) -> BoxFuture<SpotifyResult<Vec<AlbumDescription>>> {
        Box::pin(async move {
            let page = self
                .cache_get_or_write(
                    SpotCacheKey::SavedAlbums(offset, limit),
                    || self.client.get_saved_albums(offset, limit),
                    CacheExpiry::expire_in_seconds(3600),
                )
                .await?
                .deserialize()?;

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
            let page = self
                .cache_get_or_write(
                    SpotCacheKey::SavedPlaylists(offset, limit),
                    || self.client.get_saved_playlists(offset, limit),
                    CacheExpiry::expire_in_seconds(3600),
                )
                .await?
                .deserialize()?;

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
                .cache_get_or_write(
                    SpotCacheKey::Album(&id),
                    || self.client.get_album(&id),
                    CacheExpiry::expire_in_hours(24),
                )
                .await?
                .deserialize()?;

            let liked = self
                .cache_get_or_write(
                    SpotCacheKey::AlbumLiked(&id),
                    || self.client.is_album_saved(&id),
                    CacheExpiry::expire_in_hours(2),
                )
                .await?
                .deserialize()?;

            let mut album: AlbumDescription = album.into();
            album.is_liked = liked[0];

            Ok(album)
        })
    }

    fn save_album(&self, id: &str) -> BoxFuture<SpotifyResult<AlbumDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired(&SpotCacheKey::AlbumLiked(&id).into_raw())
                .await
                .unwrap_or(());
            self.cache
                .set_expired_pattern("net", &*ME_ALBUMS_CACHE)
                .await
                .unwrap_or(());
            self.client.save_album(&id).await?;
            self.get_album(&id[..]).await
        })
    }

    fn remove_saved_album(&self, id: &str) -> BoxFuture<SpotifyResult<()>> {
        let id = id.to_owned();

        Box::pin(async move {
            self.cache
                .set_expired(&SpotCacheKey::AlbumLiked(&id).into_raw())
                .await
                .unwrap_or(());
            self.cache
                .set_expired_pattern("net", &*ME_ALBUMS_CACHE)
                .await
                .unwrap_or(());
            self.client.remove_saved_album(&id).await
        })
    }

    fn get_playlist(&self, id: &str) -> BoxFuture<SpotifyResult<PlaylistDescription>> {
        let id = id.to_owned();

        Box::pin(async move {
            let playlist = self
                .cache_get_or_write(
                    SpotCacheKey::Playlist(&id),
                    || self.client.get_playlist(&id),
                    CacheExpiry::expire_in_hours(6),
                )
                .await?
                .deserialize()?;

            let mut playlist: PlaylistDescription = playlist.into();
            let mut tracks: Vec<SongDescription> = vec![];

            let mut offset = 0u32;
            let limit = 100u32;
            loop {
                let songs = self
                    .cache_get_or_write(
                        SpotCacheKey::PlaylistTracks(&id, offset, limit),
                        || self.client.get_playlist_tracks(&id, offset, limit),
                        CacheExpiry::expire_in_hours(6),
                    )
                    .await?
                    .deserialize()?;

                let mut songs: Vec<SongDescription> = songs.into();

                let songs_loaded = songs.len() as u32;
                tracks.append(&mut songs);

                if songs_loaded < limit {
                    break;
                }

                offset += limit;
            }

            playlist.songs = tracks;
            Ok(playlist)
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
            let albums = self
                .cache_get_or_write(
                    SpotCacheKey::ArtistAlbums(&id, offset, limit),
                    || self.client.get_artist_albums(&id, offset, limit),
                    CacheExpiry::expire_in_hours(24),
                )
                .await?
                .deserialize()?;

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
            let artist = self
                .cache_get_or_write(
                    SpotCacheKey::Artist(&id),
                    || self.client.get_artist(&id),
                    CacheExpiry::expire_in_hours(24),
                )
                .await?
                .deserialize()?;

            let albums = self.get_artist_albums(&id, 0, 20).await?;

            let top_tracks = self
                .cache_get_or_write(
                    SpotCacheKey::ArtistTopTracks(&id),
                    || self.client.get_artist_top_tracks(&id),
                    CacheExpiry::expire_in_hours(24),
                )
                .await?
                .deserialize()?;

            let top_tracks: Vec<SongDescription> = top_tracks.into();

            let result = ArtistDescription {
                id: artist.id,
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
            let results = self
                .client
                .search(query, offset, limit)
                .await?
                .deserialize()?;

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
