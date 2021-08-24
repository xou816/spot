mod api_models;
mod cached_client;
mod client;

pub mod cache;

pub use cached_client::{CachedSpotifyClient, SpotifyApiClient, SpotifyResult};
pub use client::SpotifyApiError;

pub async fn clear_user_cache() -> Option<()> {
    cache::CacheManager::new(&[])?
        .clear_cache_pattern("spot/net", &*cached_client::USER_CACHE)
        .await
        .ok()
}
