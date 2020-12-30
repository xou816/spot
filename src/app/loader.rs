use crate::backend::cache::*;
use gdk_pixbuf::{Pixbuf, PixbufLoader, PixbufLoaderExt};
use isahc::config::Configurable;
use isahc::{AsyncBody, AsyncReadResponseExt, HttpClient, Response};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::{Error, ErrorKind, Write};

struct LocalPixbufLoader<'a>(&'a PixbufLoader);

impl<'a> Write for LocalPixbufLoader<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.0
            .write(buf)
            .map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {}", e)))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.0
            .close()
            .map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {}", e)))?;
        Ok(())
    }
}

pub struct ImageLoader {
    cache: CacheManager,
}

impl ImageLoader {
    pub fn new() -> Self {
        Self {
            cache: CacheManager::new(&["img"]).unwrap(),
        }
    }

    fn resource_for(url: &str, ext: &str) -> String {
        let mut hasher = DefaultHasher::new();
        hasher.write(url.as_bytes());
        let hashed = hasher.finish().to_string();
        format!("img/{}.{}", hashed, ext)
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

        match self
            .cache
            .read_cache_file(&resource[..], CachePolicy::IgnoreExpiry)
            .await
        {
            CacheFile::File(buffer) => {
                loader.write_all(&buffer[..]).ok()?;
            }
            _ => {
                if let Some(mut resp) = Self::get_image(url).await {
                    let mut buffer = vec![];
                    resp.copy_to(&mut buffer).await.ok()?;
                    loader.write_all(&buffer[..]).ok()?;
                    self.cache
                        .write_cache_file(&resource[..], &buffer[..], CacheExpiry::Never)
                        .await;
                }
            }
        };

        pixbuf_loader.close().ok()?;
        pixbuf_loader.get_pixbuf()
    }
}
