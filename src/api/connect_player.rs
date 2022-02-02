use futures::Future;
use std::sync::Arc;

use super::{SpotifyApiClient, SpotifyResult};

#[derive(Clone)]
pub struct SpotifyConnectPlayer {
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
}

impl SpotifyConnectPlayer {
    pub fn new(api: Arc<dyn SpotifyApiClient + Send + Sync>) -> Self {
        Self { api }
    }

    pub fn seek(&self, position: u32) -> impl Future<Output = SpotifyResult<()>> + '_ {
        self.api.player_seek(position as usize)
    }

    pub fn play_in_context(
        &self,
        context: String,
        position: usize,
    ) -> impl Future<Output = SpotifyResult<()>> + '_ {
        self.api.player_play_in_context(context, position)
    }

    pub fn play(&self, uri: String) -> impl Future<Output = SpotifyResult<()>> + '_ {
        self.api.player_play_no_context(vec![uri])
    }

    pub fn resume(&self) -> impl Future<Output = SpotifyResult<()>> + '_ {
        self.api.player_resume()
    }

    pub fn pause(&self) -> impl Future<Output = SpotifyResult<()>> + '_ {
        self.api.player_pause()
    }
}
