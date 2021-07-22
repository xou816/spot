use secret_service::{EncryptionType, Error, SecretService};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};

static SPOT_ATTR: &str = "spot_credentials";

fn make_attributes() -> HashMap<&'static str, &'static str> {
    let mut attributes = HashMap::new();
    attributes.insert(SPOT_ATTR, "yes");
    attributes
}

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

    pub fn retrieve() -> Result<Self, Error> {
        let service = SecretService::new(EncryptionType::Dh)?;
        let collection = service.get_default_collection()?;
        let items = collection.search_items(make_attributes())?;
        let item = items.get(0).ok_or(Error::NoResult)?.get_secret()?;
        serde_json::from_slice(&item).map_err(|_| Error::Parse)
    }

    pub fn save(&self) -> Result<(), Error> {
        let service = SecretService::new(EncryptionType::Dh)?;
        let collection = service.get_default_collection()?;
        let encoded = serde_json::to_vec(&self).unwrap();
        collection.create_item(
            "Spotify Credentials",
            make_attributes(),
            &encoded,
            true,
            "text/plain",
        )?;
        Ok(())
    }
}

pub fn logout() -> Result<(), Error> {
    let service = SecretService::new(EncryptionType::Dh)?;
    let collection = service.get_default_collection()?;
    let result = collection.search_items(make_attributes())?;

    let item = result.get(0).ok_or(Error::NoResult)?;
    item.delete()
}
