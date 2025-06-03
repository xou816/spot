use tokio::sync::RwLock;

use crate::app::credentials::Credentials;

pub struct TokenStore {
    storage: RwLock<Option<Credentials>>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(None),
        }
    }

    pub fn get_cached_blocking(&self) -> Option<Credentials> {
        self.storage.blocking_read().clone()
    }

    pub async fn get_cached(&self) -> Option<Credentials> {
        self.storage.read().await.clone()
    }

    pub async fn get(&self) -> Option<Credentials> {
        let local = self.storage.read().await.clone();
        if local.is_some() {
            return local;
        }

        match Credentials::retrieve().await {
            Ok(token) => {
                self.storage.write().await.replace(token.clone());
                Some(token)
            }
            Err(e) => {
                error!("Couldnt get token from secrets service: {e}");
                None
            }
        }
    }

    pub async fn set(&self, creds: Credentials) {
        debug!("Saving token to store...");
        if let Err(e) = creds.save().await {
            warn!("Couldnt save token to secrets service: {e}");
        }
        self.storage.write().await.replace(creds);
    }

    pub async fn clear(&self) {
        if let Err(e) = Credentials::logout().await {
            warn!("Couldnt save token to secrets service: {e}");
        }
        self.storage.write().await.take();
    }
}
