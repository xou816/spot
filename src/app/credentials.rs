use secret_service::{EncryptionType, SecretService, SsError};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::str::from_utf8;

static SPOT_ATTR: &str = "spot_credentials";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub token: String,
    pub country: String,
}

pub fn try_retrieve_credentials() -> Result<Credentials, SsError> {
    let service = SecretService::new(EncryptionType::Dh)?;
    let collection = service.get_default_collection()?;
    let attributes = vec![(SPOT_ATTR, "yes")];
    let result = collection.search_items(attributes)?;

    let item = result.get(0).ok_or(SsError::NoResult)?.get_secret()?;
    let raw = from_utf8(&item).unwrap().to_string();
    let parsed = from_str(&raw).map_err(|_| SsError::Parse)?;

    Ok(parsed)
}

pub fn save_credentials(creds: Credentials) -> Result<SecretService, SsError> {
    let service = SecretService::new(EncryptionType::Dh)?;
    let collection = service.get_default_collection()?;
    let encoded = to_string(&creds).unwrap();

    collection.create_item(
        "Spotify Credentials",
        vec![(SPOT_ATTR, "yes")],
        encoded.as_bytes(),
        true,
        "text/plain",
    )?;

    Ok(service)
}

pub fn logout() -> Result<(), SsError> {
    let service = SecretService::new(EncryptionType::Dh)?;
    let collection = service.get_default_collection()?;
    let attributes = vec![(SPOT_ATTR, "yes")];
    let result = collection.search_items(attributes)?;

    let item = result.get(0).ok_or(SsError::NoResult)?;
    item.delete()
}
