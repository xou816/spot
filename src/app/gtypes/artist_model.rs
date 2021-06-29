#![allow(clippy::all)]

use gio::prelude::*;
use glib::subclass::prelude::*;

glib::wrapper! {
    pub struct ArtistModel(ObjectSubclass<imp::ArtistModel>);
}

// Constructor for new instances. This simply calls glib::Object::new() with
// initial values for our two properties and then returns the new instance
impl ArtistModel {
    pub fn new(artist: &str, image: &Option<String>, id: &str) -> ArtistModel {
        glib::Object::new::<ArtistModel>(&[("artist", &artist), ("image", image), ("id", &id)])
            .expect("Failed to create")
    }

    pub fn image_url(&self) -> Option<String> {
        self.property("image")
            .unwrap()
            .get::<&str>()
            .ok()
            .map(|s| s.to_string())
    }

    pub fn id(&self) -> Option<String> {
        self.property("id")
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
        static ref PROPERTIES: [glib::ParamSpec; 3] = [
            glib::ParamSpec::new_string("artist", "Artist", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string("image", "Image", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string("id", "id", "", None, glib::ParamFlags::READWRITE),
        ];
    }

    // This is the struct containing all state carried with
    // the new type. Generally this has to make use of
    // interior mutability.
    pub struct ArtistModel {
        artist: RefCell<Option<String>>,
        image: RefCell<Option<String>>,
        id: RefCell<Option<String>>,
    }

    // ObjectSubclass is the trait that defines the new type and
    // contains all information needed by the GObject type system,
    // including the new type's name, parent type, etc.
    #[glib::object_subclass]
    impl ObjectSubclass for ArtistModel {
        // This type name must be unique per process.
        const NAME: &'static str = "ArtistModel";

        type Type = super::ArtistModel;

        // The parent type this one is inheriting from.
        type ParentType = glib::Object;

        // Interfaces this type implements
        type Interfaces = ();

        // Called every time a new instance is created. This should return
        // a new instance of our type with its basic values.
        fn new() -> Self {
            Self {
                artist: RefCell::new(None),
                image: RefCell::new(None),
                id: RefCell::new(None),
            }
        }
    }

    // Trait that is used to override virtual methods of glib::Object.
    impl ObjectImpl for ArtistModel {
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
                "artist" => {
                    let artist = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.artist.replace(artist);
                }
                "image" => {
                    let image = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.image.replace(image);
                }
                "id" => {
                    let id = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.id.replace(id);
                }
                _ => unimplemented!(),
            }
        }

        // Called whenever a property is retrieved from this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "artist" => self.artist.borrow().to_value(),
                "image" => self.image.borrow().to_value(),
                "id" => self.id.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        // Called right after construction of the instance.
        fn constructed(&self, obj: &Self::Type) {
            // Chain up to the parent type's implementation of this virtual
            // method.
            self.parent_constructed(obj);

            // And here we could do our own initialization.
        }
    }
}
