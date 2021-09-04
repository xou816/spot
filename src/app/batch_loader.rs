use std::sync::Arc;

use crate::api::SpotifyApiClient;
use crate::app::models::*;

#[derive(Clone)]
pub struct BatchLoader {
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
}

#[derive(Clone, Debug)]
pub enum SongsSource {
    Playlist(String),
    Album(String),
}

impl PartialEq for SongsSource {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Playlist(l), Self::Playlist(r)) => l == r,
            (Self::Album(l), Self::Album(r)) => l == r,
            _ => false,
        }
    }
}

impl Eq for SongsSource {}

#[derive(Debug)]
pub struct BatchQuery {
    pub source: SongsSource,
    pub batch: Batch,
}

impl BatchQuery {
    pub fn next(&self) -> Option<Self> {
        let Self { source, batch } = self;
        Some(Self {
            source: source.clone(),
            batch: batch.next()?,
        })
    }
}

impl BatchLoader {
    pub fn new(api: Arc<dyn SpotifyApiClient + Send + Sync>) -> Self {
        Self { api }
    }

    pub async fn query(&self, query: BatchQuery) -> Option<SongBatch> {
        let api = Arc::clone(&self.api);

        match query.source {
            SongsSource::Playlist(id) => {
                let Batch {
                    offset, batch_size, ..
                } = query.batch;
                api.get_playlist_tracks(&id, offset, batch_size).await.ok()
            }
            _ => None,
        }
    }
}

impl From<&AlbumDescription> for BatchQuery {
    fn from(album: &AlbumDescription) -> Self {
        BatchQuery {
            source: SongsSource::Album(album.id.clone()),
            batch: album.last_batch,
        }
    }
}

impl From<&AlbumDescription> for SongBatch {
    fn from(album: &AlbumDescription) -> Self {
        SongBatch {
            songs: album.songs.clone(),
            batch: album.last_batch,
        }
    }
}
