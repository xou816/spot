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

    pub fn play(&self, uri: String) -> impl Future<Output = SpotifyResult<()>> + '_ {
        self.api.player_play(Some(uri))
    }

    pub fn resume(&self) -> impl Future<Output = SpotifyResult<()>> + '_ {
        self.api.player_play(None)
    }

    pub fn pause(&self) -> impl Future<Output = SpotifyResult<()>> + '_ {
        self.api.player_pause()
    }
}
