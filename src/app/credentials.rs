use std::str::from_utf8;
use secret_service::{SecretService, EncryptionType, SsError};
use serde_json::{to_string, from_str};
use serde::{Serialize, Deserialize};

static SPOT_ATTR: &'static str = "spot_credentials";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub token: String,
    pub country: String
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
        "text/plain")?;

    Ok(service)
}
