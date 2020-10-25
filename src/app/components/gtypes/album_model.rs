use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::{glib_wrapper, glib_object_wrapper};

glib_wrapper! {
    pub struct AlbumModel(Object<subclass::simple::InstanceStruct<imp::AlbumModel>, subclass::simple::ClassStruct<imp::AlbumModel>, AlbumModelClass>);

    match fn {
        get_type => || imp::AlbumModel::get_type().to_glib(),
    }
}

// Constructor for new instances. This simply calls glib::Object::new() with
// initial values for our two properties and then returns the new instance
impl AlbumModel {
    pub fn new(artist: &str, album: &str, cover_url: &str, uri: &str) -> AlbumModel {
        glib::Object::new(Self::static_type(), &[("artist", &artist), ("album", &album), ("cover_url", &cover_url), ("uri", &uri)])
            .expect("Failed to create")
            .downcast()
            .expect("Created with wrong type")
    }

    pub fn cover_url(&self) -> Option<String> {
        self.get_property("cover_url").unwrap().get::<&str>()
            .unwrap().map(|s| s.to_string())
    }

    pub fn uri(&self) -> Option<String> {
        self.get_property("uri").unwrap().get::<&str>()
            .unwrap().map(|s| s.to_string())
    }
}

mod imp {

    use super::*;
    use glib::{glib_object_subclass, glib_object_impl};

    use std::cell::RefCell;


    // Static array for defining the properties of the new type.
    static PROPERTIES: [subclass::Property; 4] = [
        subclass::Property("artist", |artist| {
            glib::ParamSpec::string(
                artist,
                "Artist",
                "Artist",
                None,
                glib::ParamFlags::READWRITE,
            )
        }),
        subclass::Property("album", |album| {
            glib::ParamSpec::string(
                album,
                "Album",
                "Album",
                None,
                glib::ParamFlags::READWRITE,
            )
        }),
        subclass::Property("cover_url", |cover_url| {
            glib::ParamSpec::string(
                cover_url,
                "Cover",
                "Cover",
                None,
                glib::ParamFlags::READWRITE,
            )
        }),
        subclass::Property("uri", |uri| {
            glib::ParamSpec::string(
                uri,
                "URI",
                "URI",
                None,
                glib::ParamFlags::READWRITE,
            )
        })
    ];

    // This is the struct containing all state carried with
    // the new type. Generally this has to make use of
    // interior mutability.
    pub struct AlbumModel {
        album: RefCell<Option<String>>,
        artist: RefCell<Option<String>>,
        uri: RefCell<Option<String>>,
        cover_url: RefCell<Option<String>>
    }

    // ObjectSubclass is the trait that defines the new type and
    // contains all information needed by the GObject type system,
    // including the new type's name, parent type, etc.
    impl ObjectSubclass for AlbumModel {
        // This type name must be unique per process.
        const NAME: &'static str = "AlbumModel";

        // The parent type this one is inheriting from.
        type ParentType = glib::Object;

        // The C/FFI instance and class structs. The simple ones
        // are enough in most cases and more is only needed to
        // expose public instance fields to C APIs or to provide
        // new virtual methods for subclasses of this type.
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        // This macro defines some boilerplate.
        glib_object_subclass!();

        // Called right before the first time an instance of the new
        // type is created. Here class specific settings can be performed,
        // including installation of properties and registration of signals
        // for the new type.
        fn class_init(klass: &mut subclass::simple::ClassStruct<Self>) {
            klass.install_properties(&PROPERTIES);
        }

        // Called every time a new instance is created. This should return
        // a new instance of our type with its basic values.
        fn new() -> Self {
            Self {
                album: RefCell::new(None),
                artist: RefCell::new(None),
                uri: RefCell::new(None),
                cover_url: RefCell::new(None),
            }
        }
    }

    // Trait that is used to override virtual methods of glib::Object.
    impl ObjectImpl for AlbumModel {
        // This macro defines some boilerplate.
        glib_object_impl!();

        // Called whenever a property is set on this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("album", ..) => {
                    let album = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.album.replace(album);
                },
                subclass::Property("artist", ..) => {
                    let artist = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.artist.replace(artist);
                },
                subclass::Property("uri", ..) => {
                    let uri = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.uri.replace(uri);
                },
                subclass::Property("cover_url", ..) => {
                    let cover_url = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.cover_url.replace(cover_url);
                }
                _ => unimplemented!(),
            }
        }

        // Called whenever a property is retrieved from this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("album", ..) => Ok(self.album.borrow().to_value()),
                subclass::Property("artist", ..) => Ok(self.artist.borrow().to_value()),
                subclass::Property("uri", ..) => Ok(self.uri.borrow().to_value()),
                subclass::Property("cover_url", ..) => Ok(self.cover_url.borrow().to_value()),
                _ => unimplemented!(),
            }
        }

        // Called right after construction of the instance.
        fn constructed(&self, obj: &glib::Object) {
            // Chain up to the parent type's implementation of this virtual
            // method.
            self.parent_constructed(obj);

            // And here we could do our own initialization.
        }
    }
}
