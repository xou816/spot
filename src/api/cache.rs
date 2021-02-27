use async_std::fs;
use async_std::io;
use async_std::path::PathBuf;
use async_std::prelude::*;
use regex::Regex;
use std::convert::{From, TryInto};
use std::future::Future;
use std::time::{Duration, SystemTime};

pub enum CacheFile {
    File(Vec<u8>),
    Expired,
    None,
}

impl From<Option<Vec<u8>>> for CacheFile {
    fn from(opt: Option<Vec<u8>>) -> CacheFile {
        match opt {
            Some(buffer) => CacheFile::File(buffer),
            None => CacheFile::None,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum CachePolicy {
    Default,
    IgnoreExpiry,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CacheExpiry {
    Never,
    AtUnixTimestamp(Duration),
}

impl CacheExpiry {
    pub fn expire_in_seconds(seconds: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        Self::AtUnixTimestamp(timestamp + Duration::new(seconds, 0))
    }

    pub fn expire_in_hours(hours: u32) -> Self {
        Self::expire_in_seconds(hours as u64 * 3600)
    }
}

#[derive(Clone)]
pub struct CacheManager {
    root: PathBuf,
}

impl CacheManager {
    pub fn new(dirs: &[&str]) -> Option<Self> {
        let root: PathBuf = glib::get_user_cache_dir()?.into();
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
    async fn read_expiry_file(&self, resource: &str) -> io::Result<Duration> {
        let expiry_file = self.cache_meta_path(resource);
        let buffer = fs::read(&expiry_file).await?;
        let slice: Box<[u8; core::mem::size_of::<u64>()]> = buffer
            .into_boxed_slice()
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "conversion error"))?;
        Ok(Duration::new(u64::from_be_bytes(*slice), 0))
    }

    async fn is_file_expired(&self, resource: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        match self.read_expiry_file(resource).await {
            Err(err) if err.kind() == io::ErrorKind::NotFound => false,
            Err(_) => true,
            Ok(expiry) => now > expiry,
        }
    }

    pub async fn read_cache_file(&self, resource: &str, policy: CachePolicy) -> CacheFile {
        let path = self.cache_path(resource);

        match policy {
            CachePolicy::IgnoreExpiry => match fs::read(&path).await {
                Err(_) => CacheFile::None,
                Ok(buf) => CacheFile::File(buf),
            },
            CachePolicy::Default => {
                let expired = self.is_file_expired(resource).await;
                if expired {
                    println!("Expired: {}", resource);
                    CacheFile::Expired
                } else {
                    match fs::read(&path).await {
                        Err(_) => CacheFile::None,
                        Ok(buf) => CacheFile::File(buf),
                    }
                }
            }
        }
    }
}

impl CacheManager {
    async fn set_expiry_for_path(&self, path: &PathBuf, expiry: Duration) -> io::Result<()> {
        let content = expiry.as_secs().to_be_bytes().to_owned();
        fs::write(path, content).await
    }

    pub async fn set_expiry_for(&self, resource: &str, expiry: Duration) -> io::Result<()> {
        let meta_file = self.cache_meta_path(resource);
        self.set_expiry_for_path(&meta_file, expiry).await
    }

    pub async fn set_expired(&self, resource: &str) -> io::Result<()> {
        self.set_expiry_for(resource, Duration::new(0, 0)).await
    }

    pub async fn set_expired_pattern(&self, dir: &str, regex: &Regex) -> io::Result<()> {
        let dir_path = self.cache_path(dir);

        let mut entries = fs::read_dir(dir_path).await?;
        while let Some(Ok(entry)) = entries.next().await {
            let matches = entry
                .file_name()
                .to_str()
                .map(|s| regex.is_match(s))
                .unwrap_or(false);
            if matches {
                self.set_expiry_for_path(&entry.path(), Duration::new(0, 0))
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
    ) -> io::Result<()> {
        let file = self.cache_path(resource);
        fs::write(&file, content).await?;

        if let CacheExpiry::AtUnixTimestamp(ts) = expiry {
            self.set_expiry_for(resource, ts).await?;
        }

        Ok(())
    }
}

pub struct CacheRequest<'a, S> {
    cache: &'a CacheManager,
    resource: S,
    policy: CachePolicy,
}

impl<'a, S> CacheRequest<'a, S>
where
    S: AsRef<str> + 'a,
{
    pub fn for_resource(cache: &'a CacheManager, resource: S, policy: CachePolicy) -> Self {
        Self {
            cache,
            resource,
            policy,
        }
    }

    pub async fn get(&self) -> Option<String> {
        match self
            .cache
            .read_cache_file(self.resource.as_ref(), self.policy)
            .await
        {
            CacheFile::File(buffer) => String::from_utf8(buffer).ok(),
            _ => None,
        }
    }

    pub async fn get_or_write<O, F, E>(&self, fresh: F, expiry: CacheExpiry) -> Result<String, E>
    where
        O: Future<Output = Result<String, E>>,
        F: FnOnce() -> O,
        E: From<io::Error>,
    {
        match self.get().await {
            Some(text) => Ok(text),
            None => {
                let fresh = fresh().await?;
                self.cache
                    .write_cache_file(self.resource.as_ref(), fresh.as_bytes(), expiry)
                    .await?;
                Ok(fresh)
            }
        }
    }
}
