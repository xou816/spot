use crate::api::cache::*;
use gdk_pixbuf::traits::PixbufLoaderExt;
use gdk_pixbuf::{Pixbuf, PixbufLoader};
use isahc::config::Configurable;
use isahc::{AsyncBody, AsyncReadResponseExt, HttpClient, Response};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::{Error, ErrorKind, Write};

// A wrapper to be able to implement the Write trait on a PixbufLoader
struct LocalPixbufLoader<'a>(&'a PixbufLoader);

impl<'a> Write for LocalPixbufLoader<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.0
            .write(buf)
            .map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {e}")))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.0
            .close()
            .map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {e}")))?;
        Ok(())
    }
}

// A helper to load remote images, with simple cache management
pub struct ImageLoader {
    cache: CacheManager,
}

impl ImageLoader {
    pub fn new() -> Self {
        Self {
            cache: CacheManager::for_dir("spot/img").unwrap(),
        }
    }

    // Downloaded images are simply named [hash of url].[file extension]
    fn resource_for(url: &str, ext: &str) -> String {
        let mut hasher = DefaultHasher::new();
        hasher.write(url.as_bytes());
        let hashed = hasher.finish().to_string();
        hashed + "." + ext
    }

    async fn get_image(url: &str) -> Option<Response<AsyncBody>> {
        let mut builder = HttpClient::builder();
        if cfg!(debug_assertions) {
            builder = builder.ssl_options(isahc::config::SslOption::DANGER_ACCEPT_INVALID_CERTS);
        }
        let client = builder.build().unwrap();
        client.get_async(url).await.ok()
    }

    pub async fn load_remote(
        &self,
        url: &str,
        ext: &str,
        width: i32,
        height: i32,
    ) -> Option<Pixbuf> {
        let resource = Self::resource_for(url, ext);
        let pixbuf_loader = PixbufLoader::new();
        pixbuf_loader.set_size(width, height);
        let mut loader = LocalPixbufLoader(&pixbuf_loader);

        // Try to read from cache first, ignoring possible expiry
        match self
            .cache
            .read_cache_file(&resource[..], CachePolicy::IgnoreExpiry)
            .await
        {
            // Write content of cache file to the pixbuf loader if the cache contained something
            Ok(CacheFile::Fresh(buffer, _)) => {
                loader.write_all(&buffer[..]).ok()?;
            }
            // Otherwise, get image over HTTP
            _ => {
                if let Some(mut resp) = Self::get_image(url).await {
                    let mut buffer = vec![];
                    // Copy the image to a buffer...
                    resp.copy_to(&mut buffer).await.ok()?;
                    // ... copy the buffer to the loader...
                    loader.write_all(&buffer[..]).ok()?;
                    // ... but also save that buffer to cache
                    self.cache
                        .write_cache_file(&resource[..], &buffer[..], CacheExpiry::Never)
                        .await
                        .ok()?;
                }
            }
        };

        pixbuf_loader.close().ok()?;
        pixbuf_loader.pixbuf()
    }
}
