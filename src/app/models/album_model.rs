#![allow(clippy::all)]

use gio::prelude::*;
use glib::subclass::prelude::*;
use glib::Properties;

// UI model!
// Despite the name, it can represent a playlist as well
glib::wrapper! {
    pub struct AlbumModel(ObjectSubclass<imp::AlbumModel>);
}

impl AlbumModel {
    pub fn new(
        artist: &String,
        album: &String,
        year: Option<u32>,
        cover: Option<&String>,
        uri: &String,
    ) -> AlbumModel {
        let year = &year.unwrap_or(0);
        glib::Object::builder()
            .property("artist", artist)
            .property("album", album)
            .property("year", year)
            .property("cover", &cover)
            .property("uri", uri)
            .build()
    }
}

mod imp {

    use super::*;

    use std::cell::{Cell, RefCell};
    use std::convert::TryFrom;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::AlbumModel)]
    pub struct AlbumModel {
        #[property(get, set)]
        album: RefCell<String>,
        #[property(get, set)]
        artist: RefCell<String>,
        #[property(get, set)]
        year: Cell<u32>,
        #[property(get, set)]
        cover: RefCell<Option<String>>,
        #[property(get, set)]
        uri: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumModel {
        const NAME: &'static str = "AlbumModel";
        type Type = super::AlbumModel;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AlbumModel {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}
