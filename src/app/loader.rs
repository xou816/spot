use std::io::{Error, ErrorKind, Write};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::rc::Rc;
use std::cell::RefCell;

use gdk_pixbuf::{Pixbuf, PixbufLoaderExt, PixbufLoader};
use isahc::ResponseExt;
use crate::backend::cache::*;

struct PixbufLoaderWithCopy<'a> {
    loader: &'a PixbufLoader,
    copy: Rc<RefCell<Vec<u8>>>
}

impl<'a> PixbufLoaderWithCopy<'a> {
    fn new(loader: &'a PixbufLoader, copy: Rc<RefCell<Vec<u8>>>) -> Self {
        Self { loader, copy }
    }
}

impl<'a> Write for PixbufLoaderWithCopy<'a> {

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.loader.write(buf).map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {}", e)))?;
        self.copy.borrow_mut().write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.loader.close().map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {}", e)))?;
        Ok(())
    }
}


pub struct ImageLoader {
    cache: CacheManager
}

impl ImageLoader {

    pub fn new() -> Self {
        Self { cache: CacheManager::new() }
    }

    fn resource_for(url: &str, ext: &str) -> String {
        let mut hasher = DefaultHasher::new();
        hasher.write(url.as_bytes());
        let hashed = hasher.finish().to_string();
        format!("{}.{}", hashed, ext)
    }

    pub async fn load_remote(&self, url: &str, ext: &str, width: i32, height: i32) -> Option<Pixbuf> {

        let resource = Self::resource_for(url, ext);
        let pixbuf_loader = PixbufLoader::new();
        pixbuf_loader.set_size(width, height);
        let copy = Rc::new(RefCell::new(Vec::new()));
        let mut writable_loader = PixbufLoaderWithCopy::new(&pixbuf_loader, Rc::clone(&copy));

        match self.cache.read_cache_file(&resource[..], CachePolicy::IgnoreExpiry).await {
            CacheFile::File(buffer) =>  {
                writable_loader.write_all(&buffer[..]).ok()?;
            },
            _ => {
                let mut resp = isahc::get_async(url).await.ok()?;
                // copy_to moves the impl Write, so I had to build something with a shared access to a secondary buffer...
                resp.copy_to(writable_loader).ok()?;
                self.cache.write_cache_file(&resource[..], copy.borrow().as_ref()).await?;
            }
        };

        pixbuf_loader.close().ok()?;
        pixbuf_loader.get_pixbuf()
    }
}
