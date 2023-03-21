use secret_service::{EncryptionType, Error, SecretService};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};

static SPOT_ATTR: &str = "spot_credentials";

// I'm not sure this is the right way to make credentials identifiable, but hey, it works
fn make_attributes() -> HashMap<&'static str, &'static str> {
    let mut attributes = HashMap::new();
    attributes.insert(SPOT_ATTR, "yes");
    attributes
}

// A (statically accessed) wrapper around the DBUS Secret Service
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub token: String,
    pub token_expiry_time: Option<SystemTime>,
    pub country: String,
}

impl Credentials {
    pub fn token_expired(&self) -> bool {
        match self.token_expiry_time {
            Some(v) => SystemTime::now() > v,
            None => true,
        }
    }

    pub async fn retrieve() -> Result<Self, Error> {
        let service = SecretService::connect(EncryptionType::Dh).await?;
        let collection = service.get_default_collection().await?;
        if collection.is_locked().await? {
            collection.unlock().await?;
        }
        let items = collection.search_items(make_attributes()).await?;
        let item = items.get(0).ok_or(Error::NoResult)?.get_secret().await?;
        serde_json::from_slice(&item).map_err(|_| Error::Unavailable)
    }

    // Try to clear the credentials
    pub async fn logout() -> Result<(), Error> {
        let service = SecretService::connect(EncryptionType::Dh).await?;
        let collection = service.get_default_collection().await?;
        if !collection.is_locked().await? {
            let result = collection.search_items(make_attributes()).await?;
            let item = result.get(0).ok_or(Error::NoResult)?;
            item.delete().await
        } else {
            warn!("Keyring is locked -- not clearing credentials");
            Ok(())
        }
    }

    pub async fn save(&self) -> Result<(), Error> {
        let service = SecretService::connect(EncryptionType::Dh).await?;
        let collection = service.get_default_collection().await?;
        if collection.is_locked().await? {
            collection.unlock().await?;
        }
        // We simply write our stuct as JSON and send it
        let encoded = serde_json::to_vec(&self).unwrap();
        collection
            .create_item(
                "Spotify Credentials",
                make_attributes(),
                &encoded,
                true,
                "text/plain",
            )
            .await?;
        Ok(())
    }
}
