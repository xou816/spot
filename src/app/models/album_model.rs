#![allow(clippy::all)]

use gio::prelude::*;
use glib::subclass::prelude::*;

glib::wrapper! {
    pub struct AlbumModel(ObjectSubclass<imp::AlbumModel>);
}

// Constructor for new instances. This simply calls glib::Object::new() with
// initial values for our two properties and then returns the new instance
impl AlbumModel {
    pub fn new(
        artist: &String,
        album: &String,
        year: Option<u32>,
        cover: Option<&String>,
        uri: &String,
    ) -> AlbumModel {
        let year = &year.unwrap_or(0);
        glib::Object::new::<AlbumModel>(&[
            ("artist", artist),
            ("album", album),
            ("year", year),
            ("cover", &cover),
            ("uri", uri),
        ])
        .expect("Failed to create")
    }

    pub fn year(&self) -> Option<u32> {
        match self.property::<u32>("year") {
            0 => None,
            year => Some(year),
        }
    }

    pub fn cover_url(&self) -> Option<String> {
        self.property("cover")
    }

    pub fn uri(&self) -> String {
        self.property("uri")
    }

    pub fn album_title(&self) -> String {
        self.property("album")
    }
}

mod imp {

    use super::*;

    use std::cell::{Cell, RefCell};

    // Static array for defining the properties of the new type.
    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 5] = [
            glib::ParamSpecString::new("artist", "Artist", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpecString::new("album", "Album", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpecUInt::new("year", "Year", "", 0, 9999, 0, glib::ParamFlags::READWRITE),
            glib::ParamSpecString::new("cover", "Cover", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpecString::new("uri", "URI", "", None, glib::ParamFlags::READWRITE),
        ];
    }

    // This is the struct containing all state carried with
    // the new type. Generally this has to make use of
    // interior mutability.
    #[derive(Default)]
    pub struct AlbumModel {
        album: RefCell<String>,
        artist: RefCell<String>,
        year: Cell<u32>,
        cover: RefCell<Option<String>>,
        uri: RefCell<String>,
    }

    // ObjectSubclass is the trait that defines the new type and
    // contains all information needed by the GObject type system,
    // including the new type's name, parent type, etc.
    #[glib::object_subclass]
    impl ObjectSubclass for AlbumModel {
        // This type name must be unique per process.
        const NAME: &'static str = "AlbumModel";

        type Type = super::AlbumModel;

        // The parent type this one is inheriting from.
        type ParentType = glib::Object;
    }

    // Trait that is used to override virtual methods of glib::Object.
    impl ObjectImpl for AlbumModel {
        fn properties() -> &'static [glib::ParamSpec] {
            &*PROPERTIES
        }

        // Called whenever a property is set on this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "album" => {
                    let album = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.album.replace(album);
                }
                "artist" => {
                    let artist = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.artist.replace(artist);
                }
                "year" => {
                    let year = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.year.replace(year);
                }
                "cover" => {
                    let cover: Option<String> = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.cover.replace(cover);
                }
                "uri" => {
                    let uri = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.uri.replace(uri);
                }
                _ => unimplemented!(),
            }
        }

        // Called whenever a property is retrieved from this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "album" => self.album.borrow().to_value(),
                "artist" => self.artist.borrow().to_value(),
                "year" => self.year.get().to_value(),
                "cover" => self.cover.borrow().to_value(),
                "uri" => self.uri.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
