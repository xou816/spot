use std::io::{Error, ErrorKind, Write};

use gdk_pixbuf::{Pixbuf, PixbufLoaderExt, PixbufLoader};
use isahc::prelude::*;
use isahc::ResponseExt;

struct LocalPixbuf<'a>(&'a PixbufLoader);

impl<'a> Write for LocalPixbuf<'a> {

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.0.write(buf).map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {}", e)))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.0.close().map_err(|e| Error::new(ErrorKind::Other, format!("glib error: {}", e)))?;
        Ok(())
    }
}

pub async fn load_remote_image(url: &str, width: i32, height: i32) -> Option<Pixbuf> {
    let pixbuf_loader = PixbufLoader::new();
    pixbuf_loader.set_size(width, height);
    let mut resp = isahc::get_async(url).await.ok()?;
    resp.copy_to(LocalPixbuf(&pixbuf_loader)).ok()?;
    pixbuf_loader.close().unwrap();
    pixbuf_loader.get_pixbuf()
}
