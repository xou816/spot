use secret_service::{EncryptionType, Error, SecretService};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::str::from_utf8;

static SPOT_ATTR: &str = "spot_credentials";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub token: String,
    pub country: String,
}

fn make_attributes() -> HashMap<&'static str, &'static str> {
    let mut attributes = HashMap::new();
    attributes.insert(SPOT_ATTR, "yes");
    attributes
}

pub fn try_retrieve_credentials() -> Result<Credentials, Error> {
    let service = SecretService::new(EncryptionType::Dh)?;
    let collection = service.get_default_collection()?;
    let result = collection.search_items(make_attributes())?;

    let item = result.get(0).ok_or(Error::NoResult)?.get_secret()?;
    let raw = from_utf8(&item).unwrap().to_string();
    let parsed = from_str(&raw).map_err(|_| Error::Parse)?;

    Ok(parsed)
}

pub fn save_credentials(creds: Credentials) -> Result<(), Error> {
    let service = SecretService::new(EncryptionType::Dh)?;
    let collection = service.get_default_collection()?;
    let encoded = to_string(&creds).unwrap();

    collection.create_item(
        "Spotify Credentials",
        make_attributes(),
        encoded.as_bytes(),
        true,
        "text/plain",
    )?;

    Ok(())
}

pub fn logout() -> Result<(), Error> {
    let service = SecretService::new(EncryptionType::Dh)?;
    let collection = service.get_default_collection()?;
    let result = collection.search_items(make_attributes())?;

    let item = result.get(0).ok_or(Error::NoResult)?;
    item.delete()
}
