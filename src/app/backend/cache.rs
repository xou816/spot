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

#[derive(PartialEq)]
pub enum CachePolicy {
    Default,
    IgnoreExpiry
}

pub struct CacheManager {}

impl CacheManager {

    pub fn new() -> Self {
        Self {}
    }

    fn cache_path(&self, resource: &str) -> Option<PathBuf> {
        let cache_dir = glib::get_user_cache_dir()?;
        glib::build_filenamev(&[cache_dir.as_path(), Path::new(resource)])
    }

    fn cache_meta_path(&self, resource: &str) -> Option<PathBuf> {
        let cache_dir = glib::get_user_cache_dir()?;
        glib::build_filenamev(&[cache_dir.as_path(), Path::new(resource), Path::new(".expiry")])
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
                None => true
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

    pub async fn write_cache_file(&self, resource: &str, content: &[u8]) -> Option<()> {
        let priority = glib::PRIORITY_DEFAULT;
        let file = self.cache_file(resource)?;

        let flags = gio::FileCreateFlags::REPLACE_DESTINATION | gio::FileCreateFlags::PRIVATE;
        let stream = file.replace_async_future(None, false, flags, priority).await.ok()?;

        let bytes = glib::Bytes::from(content);
        let _ = stream.write_bytes_async_future(&bytes, priority).await.ok()?;

        stream.close_async_future(priority).await.ok()?;

        Some(())
    }
}

fn io_error_matches(err: &glib::Error, variant: gio::IOErrorEnum) -> bool {
    match err.kind::<gio::IOErrorEnum>() {
        Some(err) if err == variant => true,
        _ => false
    }
}
