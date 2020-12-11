use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::convert::TryInto;
use futures::stream::unfold;
use futures::StreamExt;

use gio::prelude::*;

pub enum CacheFile {
    File(Vec<u8>),
    Expired,
    None
}

impl From<Option<Vec<u8>>> for CacheFile {

    fn from(opt: Option<Vec<u8>>) -> CacheFile {
        match opt {
            Some(buffer) => CacheFile::File(buffer),
            None => CacheFile::None
        }
    }

}

#[derive(PartialEq, Clone, Copy)]
pub enum CachePolicy {
    Default,
    IgnoreExpiry
}


#[derive(PartialEq, Clone, Copy)]
pub enum CacheExpiry {
    Never,
    AtUnixTimestamp(Duration)
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
    root: PathBuf
}

impl CacheManager {

    pub fn new(dirs: &[&str]) -> Option<Self> {
        let root = glib::get_user_cache_dir()?;
        let root_unwrapped = root.to_str()?;
        let mask = 0o744;

        for &dir in dirs.iter() {
            let path = format!("{}/{}", root_unwrapped, dir);
            glib::mkdir_with_parents(path, mask);
        }

        Some(Self { root })
    }

    fn cache_path(&self, resource: &str) -> Option<PathBuf> {
        let cache_dir = glib::get_user_cache_dir()?;
        glib::build_filenamev(&[cache_dir.as_path(), Path::new(resource)])
    }

    fn cache_meta_path(&self, resource: &str) -> Option<PathBuf> {
        let cache_dir = glib::get_user_cache_dir()?;
        let full = format!("{}.{}", resource, "expiry");
        glib::build_filenamev(&[cache_dir.as_path(), Path::new(&full)])
    }

    fn cache_file(&self, resource: &str) -> Option<gio::File> {
        let full_path = self.cache_path(resource)?;
        Some(gio::File::new_for_path(full_path))
    }

    fn cache_meta_file(&self, resource: &str) -> Option<gio::File> {
        let full_path = self.cache_meta_path(resource)?;
        Some(gio::File::new_for_path(full_path))
    }

}

impl CacheManager {

    async fn read_chunk(stream: &gio::FileInputStream, priority: glib::Priority) -> Option<Vec<u8>> {
        let size = 64;
        let (mut buffer, read_count) = stream.read_async_future(vec![0; size], priority).await.ok()?;
        if read_count == 0 {
            None
        } else {
            buffer.truncate(read_count);
            Some(buffer)
        }
    }

    async fn read_all_chunks(stream: &gio::FileInputStream, priority: glib::Priority) -> Option<Vec<u8>> {
        let buffers = unfold((), |_| async {
            if let Some(buffer) = Self::read_chunk(&stream, priority).await {
                Some((buffer, ()))
            } else {
                Self::close(&stream, priority).await?;
                None
            }
        }).collect::<Vec<Vec<u8>>>().await;


        let full_file = buffers.iter().fold(vec![], |mut acc: Vec<u8>, buffer| {
            acc.extend(buffer);
            acc
        });


        Some(full_file)
    }

    async fn close(stream: &gio::FileInputStream, priority: glib::Priority) -> Option<()> {
        stream.close_async_future(priority).await.ok()
    }

    async fn read_timestamp(stream: &gio::FileInputStream, priority: glib::Priority) -> Option<Duration> {
        let buffer = Self::read_all_chunks(&stream, priority).await?;
        let slice: Box<[u8; core::mem::size_of::<u64>()]> = buffer.into_boxed_slice().try_into().ok()?;
        Some(Duration::new(u64::from_be_bytes(*slice), 0))
    }

    async fn is_file_expired(&self, resource: &str, priority: glib::Priority) -> bool {

        let expiry_file = self.cache_meta_file(resource).unwrap();
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

        match expiry_file.read_async_future(priority).await {
            Err(err) if io_error_matches(&err, gio::IOErrorEnum::NotFound) => false,
            Err(_) => true,
            Ok(stream) => match Self::read_timestamp(&stream, priority).await {
                Some(expiry) => now > expiry,
                None => false
            }
        }
    }

    pub async fn read_cache_file(&self, resource: &str, policy: CachePolicy) -> CacheFile {

        let priority = glib::PRIORITY_DEFAULT;
        let file = self.cache_file(resource).unwrap();

        match policy {
            CachePolicy::IgnoreExpiry => {
                match file.read_async_future(priority).await {
                    Err(_) => CacheFile::None,
                    Ok(stream) => Self::read_all_chunks(&stream, priority).await.into()
                }
            },
            CachePolicy::Default => {
                let expired = self.is_file_expired(resource, priority).await;
                if expired {
                    println!("Expired: {}", resource);
                    CacheFile::Expired
                } else {
                    match file.read_async_future(priority).await {
                        Err(_) => CacheFile::None,
                        Ok(stream) => Self::read_all_chunks(&stream, priority).await.into()
                    }
                }
            }
        }
    }
}

impl CacheManager {

    pub async fn set_expiry_for(&self, resource: &str, expiry: Duration) -> Option<()> {
        let priority = glib::PRIORITY_DEFAULT;
        let meta_file = self.cache_meta_file(resource).unwrap();
        let content = expiry.as_secs().to_be_bytes().to_owned();

        let flags = gio::FileCreateFlags::REPLACE_DESTINATION | gio::FileCreateFlags::PRIVATE;
        let stream = meta_file.replace_async_future(None, false, flags, priority).await.ok()?;

        let bytes = glib::Bytes::from(&content);
        let _ = stream.write_bytes_async_future(&bytes, priority).await.ok()?;

        stream.close_async_future(priority).await.ok()?;
        Some(())
    }

    pub async fn write_cache_file(&self, resource: &str, content: &[u8], expiry: CacheExpiry) -> Option<()> {
        let priority = glib::PRIORITY_DEFAULT;
        let file = self.cache_file(resource)?;

        let flags = gio::FileCreateFlags::REPLACE_DESTINATION | gio::FileCreateFlags::PRIVATE;
        let stream = file.replace_async_future(None, false, flags, priority).await.ok()?;

        let bytes = glib::Bytes::from(content);
        let _ = stream.write_bytes_async_future(&bytes, priority).await.ok()?;

        stream.close_async_future(priority).await.ok()?;

        if let CacheExpiry::AtUnixTimestamp(ts) = expiry {
            self.set_expiry_for(resource, ts).await?;
        }

        Some(())
    }
}

fn io_error_matches(err: &glib::Error, variant: gio::IOErrorEnum) -> bool {
    match err.kind::<gio::IOErrorEnum>() {
        Some(err) if err == variant => true,
        _ => false
    }
}
