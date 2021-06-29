use async_std::fs;
use async_std::io;
use async_std::path::PathBuf;
use async_std::prelude::*;
use core::mem::size_of;
use futures::join;
use regex::Regex;
use std::convert::From;
use std::future::Future;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("No content available")]
    NoContent,
    #[error("File could not be saved to cache: {0}")]
    WriteError(std::io::Error),
    #[error("File could not be read from cache: {0}")]
    ReadError(std::io::Error),
    #[error("File could not be removed from cache: {0}")]
    RemoveError(std::io::Error),
    #[error(transparent)]
    ConversionError(#[from] std::string::FromUtf8Error),
}

pub type ETag = String;

pub enum CacheFile {
    Fresh(Vec<u8>, Option<ETag>),
    Expired(Vec<u8>, Option<ETag>),
    None,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CachePolicy {
    Default,      // query remote cache when stale
    IgnoreExpiry, // always use cached value
    Revalidate,   // always query remote cache
    IgnoreCached, // ignore cache alltogether
}

#[derive(PartialEq, Clone, Debug)]
pub enum CacheExpiry {
    Never,
    AtUnixTimestamp(Duration, Option<ETag>),
}

impl CacheExpiry {
    pub fn expire_in_seconds(seconds: u64, etag: Option<ETag>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        Self::AtUnixTimestamp(timestamp + Duration::new(seconds, 0), etag)
    }

    fn is_expired(&self) -> bool {
        match self {
            Self::Never => false,
            Self::AtUnixTimestamp(ref duration, _) => {
                let now = &SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                now > duration
            }
        }
    }

    fn etag(&self) -> Option<&String> {
        match self {
            Self::Never => None,
            Self::AtUnixTimestamp(_, ref etag) => etag.as_ref(),
        }
    }
}

#[derive(Clone)]
pub struct CacheManager {
    root: PathBuf,
}

impl CacheManager {
    pub fn new(dirs: &[&str]) -> Option<Self> {
        let root: PathBuf = glib::user_cache_dir().into();
        let mask = 0o744;

        for &dir in dirs.iter() {
            glib::mkdir_with_parents(root.join(dir), mask);
        }

        Some(Self { root })
    }

    fn cache_path(&self, resource: &str) -> PathBuf {
        self.root.join(resource)
    }

    fn cache_meta_path(&self, resource: &str) -> PathBuf {
        let full = format!("{}.{}", resource, "expiry");
        self.root.join(&full)
    }
}

impl CacheManager {
    async fn read_expiry_file(&self, resource: &str) -> Result<CacheExpiry, CacheError> {
        let expiry_file = self.cache_meta_path(resource);
        match fs::read(&expiry_file).await {
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => Ok(CacheExpiry::Never),
                _ => Err(CacheError::ReadError(e)),
            },
            Ok(buffer) => {
                const OFFSET: usize = size_of::<u64>();

                let mut duration: [u8; OFFSET] = Default::default();
                duration.copy_from_slice(&buffer[..OFFSET]);
                let duration = Duration::from_secs(u64::from_be_bytes(duration));

                let etag = String::from_utf8((&buffer[OFFSET..]).to_vec()).ok();

                Ok(CacheExpiry::AtUnixTimestamp(duration, etag))
            }
        }
    }

    pub async fn read_cache_file(
        &self,
        resource: &str,
        policy: CachePolicy,
    ) -> Result<CacheFile, CacheError> {
        if matches!(policy, CachePolicy::IgnoreCached) {
            return Ok(CacheFile::None);
        }

        let path = self.cache_path(resource);
        let (file, expiry) = join!(fs::read(&path), self.read_expiry_file(resource));

        match (file, policy) {
            (Ok(buf), CachePolicy::IgnoreExpiry) => Ok(CacheFile::Fresh(buf, None)),
            (Ok(buf), CachePolicy::Revalidate) => {
                let expiry = expiry.unwrap_or(CacheExpiry::Never);
                let etag = expiry.etag().cloned();
                Ok(CacheFile::Expired(buf, etag))
            }
            (Ok(buf), CachePolicy::Default) => {
                let expiry = expiry?;
                let etag = expiry.etag().cloned();
                Ok(if expiry.is_expired() {
                    CacheFile::Expired(buf, etag)
                } else {
                    CacheFile::Fresh(buf, etag)
                })
            }
            (_, CachePolicy::IgnoreCached) => Ok(CacheFile::None),
            (Err(e), _) => match e.kind() {
                io::ErrorKind::NotFound => Ok(CacheFile::None),
                _ => Err(CacheError::ReadError(e)),
            },
        }
    }
}

impl CacheManager {
    async fn set_expiry_for_path(
        &self,
        path: &PathBuf,
        expiry: CacheExpiry,
    ) -> Result<(), CacheError> {
        if let CacheExpiry::AtUnixTimestamp(duration, etag) = expiry {
            let mut content = duration.as_secs().to_be_bytes().to_vec();
            if let Some(etag) = etag {
                content.append(&mut etag.into_bytes());
            }
            fs::write(path, content)
                .await
                .map_err(CacheError::WriteError)?;
        }
        Ok(())
    }

    pub async fn clear_cache_pattern(&self, dir: &str, regex: &Regex) -> Result<(), CacheError> {
        let dir_path = self.cache_path(dir);

        let mut entries = fs::read_dir(dir_path)
            .await
            .map_err(CacheError::ReadError)?;

        while let Some(Ok(entry)) = entries.next().await {
            let matches = entry
                .file_name()
                .to_str()
                .map(|s| regex.is_match(s))
                .unwrap_or(false);
            if matches {
                fs::remove_file(entry.path())
                    .await
                    .map_err(CacheError::RemoveError)?;
            }
        }

        Ok(())
    }

    pub async fn set_expired_pattern(&self, dir: &str, regex: &Regex) -> Result<(), CacheError> {
        let dir_path = self.cache_path(dir);

        let mut entries = fs::read_dir(dir_path)
            .await
            .map_err(CacheError::ReadError)?;

        while let Some(Ok(entry)) = entries.next().await {
            let matches = entry
                .file_name()
                .to_str()
                .map(|s| regex.is_match(s))
                .unwrap_or(false);
            if matches {
                self.set_expiry_for_path(&entry.path(), CacheExpiry::expire_in_seconds(0, None))
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn write_cache_file(
        &self,
        resource: &str,
        content: &[u8],
        expiry: CacheExpiry,
    ) -> Result<(), CacheError> {
        let file = self.cache_path(resource);
        let meta = self.cache_meta_path(resource);
        let (r1, r2) = join!(
            fs::write(&file, content),
            self.set_expiry_for_path(&meta, expiry)
        );
        r1.map_err(CacheError::WriteError)?;
        r2?;
        Ok(())
    }

    pub async fn get_or_write<O, F, E>(
        &self,
        resource: &str,
        policy: CachePolicy,
        fetch: F,
    ) -> Result<Vec<u8>, E>
    where
        O: Future<Output = Result<FetchResult, E>>,
        F: FnOnce(Option<ETag>) -> O,
        E: From<CacheError>,
    {
        let file = self.read_cache_file(resource, policy).await?;
        match file {
            CacheFile::Fresh(buf, _) => Ok(buf),
            CacheFile::Expired(buf, etag) => match fetch(etag).await? {
                FetchResult::NotModified(expiry) => {
                    let meta = self.cache_meta_path(resource);
                    self.set_expiry_for_path(&meta, expiry).await?;
                    Ok(buf)
                }
                FetchResult::Modified(fresh, expiry) => {
                    self.write_cache_file(resource, &fresh, expiry).await?;
                    Ok(fresh)
                }
            },
            CacheFile::None => match fetch(None).await? {
                FetchResult::NotModified(_) => Err(E::from(CacheError::NoContent)),
                FetchResult::Modified(fresh, expiry) => {
                    self.write_cache_file(resource, &fresh, expiry).await?;
                    Ok(fresh)
                }
            },
        }
    }
}

pub enum FetchResult {
    NotModified(CacheExpiry),
    Modified(Vec<u8>, CacheExpiry),
}
