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
        artist: &str,
        album: &str,
        year: Option<u32>,
        cover: &Option<String>,
        uri: &str,
    ) -> AlbumModel {
        let year = year.unwrap_or(0);
        glib::Object::new::<AlbumModel>(&[
            ("artist", &artist),
            ("album", &album),
            ("year", &year),
            ("cover", cover),
            ("uri", &uri),
        ])
        .expect("Failed to create")
    }

    pub fn year(&self) -> Option<u32> {
        match self.property("year").unwrap().get::<u32>().unwrap() {
            0 => None,
            year => Some(year),
        }
    }

    pub fn cover_url(&self) -> Option<String> {
        self.property("cover")
            .unwrap()
            .get::<&str>()
            .ok()
            .map(|s| s.to_string())
    }

    pub fn uri(&self) -> Option<String> {
        self.property("uri")
            .unwrap()
            .get::<&str>()
            .ok()
            .map(|s| s.to_string())
    }

    pub fn album_title(&self) -> Option<String> {
        self.property("album")
            .unwrap()
            .get::<&str>()
            .ok()
            .map(|s| s.to_string())
    }
}

mod imp {

    use super::*;

    use std::cell::RefCell;

    // Static array for defining the properties of the new type.
    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 5] = [
            glib::ParamSpec::new_string("artist", "Artist", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string("album", "Album", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_uint("year", "Year", "", 0, 9999, 0, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string("cover", "Cover", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string("uri", "URI", "", None, glib::ParamFlags::READWRITE),
        ];
    }

    // This is the struct containing all state carried with
    // the new type. Generally this has to make use of
    // interior mutability.
    pub struct AlbumModel {
        album: RefCell<Option<String>>,
        artist: RefCell<Option<String>>,
        year: RefCell<Option<u32>>,
        cover: RefCell<Option<String>>,
        uri: RefCell<Option<String>>,
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

        // Interfaces this type implements
        type Interfaces = ();

        // Called every time a new instance is created. This should return
        // a new instance of our type with its basic values.
        fn new() -> Self {
            Self {
                album: RefCell::new(None),
                artist: RefCell::new(None),
                year: RefCell::new(None),
                cover: RefCell::new(None),
                uri: RefCell::new(None),
            }
        }
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
                    match year {
                        0 => self.year.replace(None),
                        y => self.year.replace(Some(y)),
                    };
                }
                "cover" => {
                    let cover = value
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
                "year" => self.year.borrow().unwrap_or(0).to_value(),
                "cover" => self.cover.borrow().to_value(),
                "uri" => self.uri.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
