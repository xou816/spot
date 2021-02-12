use gio::prelude::*;
use glib::glib_wrapper;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

glib_wrapper! {
    pub struct ArtistModel(Object<subclass::simple::InstanceStruct<imp::ArtistModel>, subclass::simple::ClassStruct<imp::ArtistModel>, ArtistModelClass>);

    match fn {
        get_type => || imp::ArtistModel::get_type().to_glib(),
    }
}

// Constructor for new instances. This simply calls glib::Object::new() with
// initial values for our two properties and then returns the new instance
impl ArtistModel {
    pub fn new(artist: &str, image: &Option<String>, id: &str) -> ArtistModel {
        glib::Object::new(
            Self::static_type(),
            &[("artist", &artist), ("image", image), ("id", &id)],
        )
        .expect("Failed to create")
        .downcast()
        .expect("Created with wrong type")
    }

    pub fn image_url(&self) -> Option<String> {
        self.get_property("image")
            .unwrap()
            .get::<&str>()
            .unwrap()
            .map(|s| s.to_string())
    }

    pub fn id(&self) -> Option<String> {
        self.get_property("id")
            .unwrap()
            .get::<&str>()
            .unwrap()
            .map(|s| s.to_string())
    }
}

mod imp {

    use super::*;
    use glib::{glib_object_impl, glib_object_subclass};

    use std::cell::RefCell;

    // Static array for defining the properties of the new type.
    static PROPERTIES: [subclass::Property; 3] = [
        subclass::Property("artist", |artist| {
            glib::ParamSpec::string(
                artist,
                "Artist",
                "Artist",
                None,
                glib::ParamFlags::READWRITE,
            )
        }),
        subclass::Property("image", |image| {
            glib::ParamSpec::string(image, "Image", "Image", None, glib::ParamFlags::READWRITE)
        }),
        subclass::Property("id", |id| {
            glib::ParamSpec::string(id, "id", "id", None, glib::ParamFlags::READWRITE)
        }),
    ];

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
    impl ObjectSubclass for ArtistModel {
        // This type name must be unique per process.
        const NAME: &'static str = "ArtistModel";

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
        fn class_init(klass: &mut Self::Class) {
            klass.install_properties(&PROPERTIES);
        }

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
        // This macro defines some boilerplate.
        glib_object_impl!();

        // Called whenever a property is set on this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("artist", ..) => {
                    let artist = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.artist.replace(artist);
                }
                subclass::Property("image", ..) => {
                    let image = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.image.replace(image);
                }
                subclass::Property("id", ..) => {
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
        fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("artist", ..) => Ok(self.artist.borrow().to_value()),
                subclass::Property("image", ..) => Ok(self.image.borrow().to_value()),
                subclass::Property("id", ..) => Ok(self.id.borrow().to_value()),
                _ => unimplemented!(),
            }
        }
    }
}
