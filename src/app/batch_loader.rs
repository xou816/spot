use gettextrs::gettext;
use std::sync::Arc;

use crate::api::{SpotifyApiClient, SpotifyApiError};
use crate::app::models::*;
use crate::app::AppAction;

// A wrapper around the Spotify API to load batches of songs from various sources (see below)
#[derive(Clone)]
pub struct BatchLoader {
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
}

// The sources mentionned above
#[derive(Clone, Debug)]
pub enum SongsSource {
    Playlist(String),
    Album(String),
    SavedTracks,
}

impl PartialEq for SongsSource {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Playlist(l), Self::Playlist(r)) => l == r,
            (Self::Album(l), Self::Album(r)) => l == r,
            (Self::SavedTracks, Self::SavedTracks) => true,
            _ => false,
        }
    }
}

impl Eq for SongsSource {}

impl SongsSource {
    pub fn has_spotify_uri(&self) -> bool {
        matches!(self, Self::Playlist(_) | Self::Album(_))
    }

    pub fn spotify_uri(&self) -> Option<String> {
        match self {
            Self::Playlist(id) => Some(format!("spotify:playlist:{}", id)),
            Self::Album(id) => Some(format!("spotify:album:{}", id)),
            _ => None,
        }
    }
}

// How to query for a batch: specify a source, and a batch to get (offset + number of elements to get)
#[derive(Debug)]
pub struct BatchQuery {
    pub source: SongsSource,
    pub batch: Batch,
}

impl BatchQuery {
    // Given a query, compute the next batch to get (if any)
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

    // Query a batch and create an action when it's been retrieved succesfully
    pub async fn query<ActionCreator>(
        &self,
        query: BatchQuery,
        create_action: ActionCreator,
    ) -> Option<AppAction>
    where
        ActionCreator: FnOnce(SongsSource, SongBatch) -> AppAction,
    {
        let api = Arc::clone(&self.api);

        let Batch {
            offset, batch_size, ..
        } = query.batch;
        let result = match &query.source {
            SongsSource::Playlist(id) => api.get_playlist_tracks(id, offset, batch_size).await,
            SongsSource::SavedTracks => api.get_saved_tracks(offset, batch_size).await,
            SongsSource::Album(id) => api.get_album_tracks(id, offset, batch_size).await,
        };

        match result {
            Ok(batch) => Some(create_action(query.source, batch)),
            // No token? Why was the batch loader called? Ah, whatever
            Err(SpotifyApiError::NoToken) => None,
            Err(err) => {
                error!("Spotify API error: {}", err);
                Some(AppAction::ShowNotification(gettext(
                    // translators: This notification is the default message for unhandled errors. Logs refer to console output.
                    "An error occured. Check logs for details!",
                )))
            }
        }
    }
}
