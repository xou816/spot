use std::io::{Error, ErrorKind, Write};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::rc::Rc;
use std::cell::RefCell;

use gdk_pixbuf::{Pixbuf, PixbufLoaderExt, PixbufLoader};
use isahc::ResponseExt;
use crate::backend::cache::*;

struct WriteSpy<W: Write> {
    write_impl: W,
    spy: Rc<RefCell<Vec<u8>>>
}


impl <W> WriteSpy<W> where W: Write {

    fn new(write_impl: W) -> Self {
        Self { write_impl, spy: Rc::new(RefCell::new(Vec::new())) }
    }

    fn get_spy(&self) -> Rc<RefCell<Vec<u8>>> {
        Rc::clone(&self.spy)
    }
}

impl <W> Write for WriteSpy<W> where W: Write {

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let len = self.write_impl.write(buf)?;
        self.spy.borrow_mut().write(buf)?;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.write_impl.flush()
    }
}

struct LocalPixbufLoader<'a>(&'a PixbufLoader);


impl<'a> Write for LocalPixbufLoader<'a> {

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.0.write(buf).map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {}", e)))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.0.close().map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {}", e)))?;
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
        let mut writable_loader = WriteSpy::new(LocalPixbufLoader(&pixbuf_loader));

        match self.cache.read_cache_file(&resource[..], CachePolicy::IgnoreExpiry).await {
            CacheFile::File(buffer) =>  {
                writable_loader.write_all(&buffer[..]).ok()?;
            },
            _ => {
                let mut resp = isahc::get_async(url).await.ok()?;
                let copy = writable_loader.get_spy();
                resp.copy_to(writable_loader).ok()?;
                self.cache.write_cache_file(&resource[..], copy.borrow().as_ref(), CacheExpiry::Never).await?;
            }
        };

        pixbuf_loader.close().ok()?;
        pixbuf_loader.get_pixbuf()
    }
}
