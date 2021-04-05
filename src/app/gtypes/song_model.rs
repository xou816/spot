use gio::prelude::*;
use glib::glib_wrapper;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Value;

glib_wrapper! {
    pub struct SongModel(Object<subclass::simple::InstanceStruct<imp::SongModel>, subclass::simple::ClassStruct<imp::SongModel>, SongModelClass>);

    match fn {
        get_type => || imp::SongModel::get_type().to_glib(),
    }
}

// Constructor for new instances. This simply calls glib::Object::new() with
// initial values for our two properties and then returns the new instance
impl SongModel {
    pub fn new(id: &str, index: u32, title: &str, artist: &str, duration: &str) -> SongModel {
        glib::Object::new(
            Self::static_type(),
            &[
                ("index", &index),
                ("title", &title),
                ("artist", &artist),
                ("id", &id),
                ("duration", &duration),
            ],
        )
        .expect("Failed to create")
        .downcast()
        .expect("Created with wrong type")
    }

    pub fn set_playing(&self, is_playing: bool) {
        self.set_property("playing", &Value::from(&is_playing))
            .expect("set 'playing' failed");
    }

    pub fn set_selected(&self, is_selected: bool) {
        self.set_property("selected", &Value::from(&is_selected))
            .expect("set 'selected' failed");
    }

    pub fn get_playing(&self) -> bool {
        self.get_property("playing")
            .unwrap()
            .get::<bool>()
            .unwrap()
            .unwrap()
    }

    pub fn get_selected(&self) -> bool {
        self.get_property("selected")
            .unwrap()
            .get::<bool>()
            .unwrap()
            .unwrap()
    }

    pub fn get_id(&self) -> String {
        self.get_property("id")
            .unwrap()
            .get::<&str>()
            .unwrap()
            .unwrap()
            .to_string()
    }

    pub fn connect_playing_local<F: Fn(&Self) + 'static>(&self, handler: F) {
        self.connect_local("notify::playing", true, move |values| {
            if let Ok(Some(_self)) = values[0].get::<Self>() {
                handler(&_self);
            }
            None
        })
        .expect("connecting to prop 'playing' failed");
    }

    pub fn connect_selected_local<F: Fn(&Self) + 'static>(&self, handler: F) {
        self.connect_local("notify::selected", true, move |values| {
            if let Ok(Some(_self)) = values[0].get::<Self>() {
                handler(&_self);
            }
            None
        })
        .expect("connecting to prop 'selected' failed");
    }
}

mod imp {

    use super::*;
    use glib::{glib_object_impl, glib_object_subclass};

    use std::cell::RefCell;

    // Static array for defining the properties of the new type.
    static PROPERTIES: [subclass::Property; 7] = [
        subclass::Property("index", |index| {
            glib::ParamSpec::uint(
                index,
                "Index",
                "Index",
                1,
                u32::MAX,
                1,
                glib::ParamFlags::READWRITE,
            )
        }),
        subclass::Property("title", |title| {
            glib::ParamSpec::string(title, "Title", "Title", None, glib::ParamFlags::READWRITE)
        }),
        subclass::Property("artist", |artist| {
            glib::ParamSpec::string(
                artist,
                "Artist",
                "Artist",
                None,
                glib::ParamFlags::READWRITE,
            )
        }),
        subclass::Property("id", |id| {
            glib::ParamSpec::string(id, "id", "id", None, glib::ParamFlags::READWRITE)
        }),
        subclass::Property("duration", |duration| {
            glib::ParamSpec::string(
                duration,
                "Duration",
                "Duration",
                None,
                glib::ParamFlags::READWRITE,
            )
        }),
        subclass::Property("playing", |playing| {
            glib::ParamSpec::boolean(
                playing,
                "Playing",
                "Playing",
                false,
                glib::ParamFlags::READWRITE,
            )
        }),
        subclass::Property("selected", |playing| {
            glib::ParamSpec::boolean(
                playing,
                "Selected",
                "Selected",
                false,
                glib::ParamFlags::READWRITE,
            )
        }),
    ];

    // This is the struct containing all state carried with
    // the new type. Generally this has to make use of
    // interior mutability.
    pub struct SongModel {
        id: RefCell<Option<String>>,
        index: RefCell<u32>,
        title: RefCell<Option<String>>,
        artist: RefCell<Option<String>>,
        duration: RefCell<Option<String>>,
        playing: RefCell<bool>,
        selected: RefCell<bool>,
    }

    // ObjectSubclass is the trait that defines the new type and
    // contains all information needed by the GObject type system,
    // including the new type's name, parent type, etc.
    impl ObjectSubclass for SongModel {
        // This type name must be unique per process.
        const NAME: &'static str = "SongModel";

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
                id: RefCell::new(None),
                index: RefCell::new(1),
                title: RefCell::new(None),
                artist: RefCell::new(None),
                duration: RefCell::new(None),
                playing: RefCell::new(false),
                selected: RefCell::new(false),
            }
        }
    }

    // Trait that is used to override virtual methods of glib::Object.
    impl ObjectImpl for SongModel {
        // This macro defines some boilerplate.
        glib_object_impl!();

        // Called whenever a property is set on this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("index", ..) => {
                    let index = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`")
                        .unwrap();
                    self.index.replace(index);
                }
                subclass::Property("title", ..) => {
                    let title = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.title.replace(title);
                }
                subclass::Property("artist", ..) => {
                    let artist = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.artist.replace(artist);
                }
                subclass::Property("id", ..) => {
                    let id = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.id.replace(id);
                }
                subclass::Property("duration", ..) => {
                    let dur = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.duration.replace(dur);
                }
                subclass::Property("playing", ..) => {
                    let playing = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`")
                        .unwrap();
                    self.playing.replace(playing);
                }
                subclass::Property("selected", ..) => {
                    let selected = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`")
                        .unwrap();
                    self.selected.replace(selected);
                }
                _ => unimplemented!(),
            }
        }

        // Called whenever a property is retrieved from this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("index", ..) => Ok(self.index.borrow().to_value()),
                subclass::Property("title", ..) => Ok(self.title.borrow().to_value()),
                subclass::Property("artist", ..) => Ok(self.artist.borrow().to_value()),
                subclass::Property("id", ..) => Ok(self.id.borrow().to_value()),
                subclass::Property("duration", ..) => Ok(self.duration.borrow().to_value()),
                subclass::Property("playing", ..) => Ok(self.playing.borrow().to_value()),
                subclass::Property("selected", ..) => Ok(self.selected.borrow().to_value()),
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
