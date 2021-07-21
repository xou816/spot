use secret_service::{EncryptionType, Error, SecretService};
use serde::{de::Error as _, ser::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

static SPOT_ATTR: &str = "spot_credentials";

fn make_attributes() -> HashMap<&'static str, &'static str> {
    let mut attributes = HashMap::new();
    attributes.insert(SPOT_ATTR, "yes");
    attributes
}

mod serde_unix_timestamp {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match <Option<u64>>::deserialize(deserializer)? {
            Some(v) => Ok(Some(
                SystemTime::UNIX_EPOCH
                    .checked_add(Duration::from_secs(v))
                    .ok_or_else(|| D::Error::custom("overflow deserializing SystemTime"))?,
            )),
            None => Ok(None),
        }
    }

    pub fn serialize<S>(system_time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        system_time
            .map(|v| match v.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(v) => Ok(v.as_secs()),
                Err(_) => Err(S::Error::custom("SystemTime must be later than UNIX_EPOCH")),
            })
            .transpose()?
            .serialize(serializer)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub token: String,
    #[serde(with = "serde_unix_timestamp")]
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
