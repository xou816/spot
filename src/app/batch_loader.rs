use gettextrs::gettext;
use std::sync::Arc;

use crate::api::SpotifyApiClient;
use crate::app::models::*;
use crate::app::AppAction;

#[derive(Clone)]
pub struct BatchLoader {
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
}

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

    pub async fn query<ActionCreator>(
        &self,
        query: BatchQuery,
        create_action: ActionCreator,
    ) -> AppAction
    where
        ActionCreator: FnOnce(SongBatch) -> AppAction,
    {
        let api = Arc::clone(&self.api);

        let result = match query.source {
            SongsSource::Playlist(id) => {
                let Batch {
                    offset, batch_size, ..
                } = query.batch;
                api.get_playlist_tracks(&id, offset, batch_size).await
            }
            SongsSource::SavedTracks => {
                let Batch {
                    offset, batch_size, ..
                } = query.batch;
                api.get_saved_tracks(offset, batch_size).await
            }
            SongsSource::Album(id) => {
                let Batch {
                    offset, batch_size, ..
                } = query.batch;
                api.get_album_tracks(&id, offset, batch_size).await
            }
        };

        match result {
            Ok(batch) => create_action(batch),
            Err(err) => {
                error!("Spotify API error: {}", err);
                AppAction::ShowNotification(gettext(
                    // translators: This notification is the default message for unhandled errors. Logs refer to console output.
                    "An error occurred. Check logs for details!",
                ))
            }
        }
    }
}
