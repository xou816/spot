#![allow(clippy::all)]

use gio::prelude::*;
use glib::subclass::prelude::*;

glib::wrapper! {
    pub struct SongModel(ObjectSubclass<imp::SongModel>);
}

// Constructor for new instances. This simply calls glib::Object::new() with
// initial values for our two properties and then returns the new instance
impl SongModel {
    pub fn new(id: &str, index: u32, title: &str, artist: &str, duration: &str) -> SongModel {
        glib::Object::new::<SongModel>(&[
            ("index", &index),
            ("title", &title),
            ("artist", &artist),
            ("id", &id),
            ("duration", &duration),
        ])
        .expect("Failed to create")
    }

    pub fn set_playing(&self, is_playing: bool) {
        self.set_property("playing", is_playing)
            .expect("set 'playing' failed");
    }

    pub fn set_selected(&self, is_selected: bool) {
        self.set_property("selected", is_selected)
            .expect("set 'selected' failed");
    }

    pub fn get_playing(&self) -> bool {
        self.property("playing").unwrap().get::<bool>().unwrap()
    }

    pub fn get_selected(&self) -> bool {
        self.property("selected").unwrap().get::<bool>().unwrap()
    }

    pub fn get_id(&self) -> String {
        self.property("id")
            .unwrap()
            .get::<&str>()
            .unwrap()
            .to_string()
    }

    pub fn connect_playing_local<F: Fn(&Self) + 'static>(&self, handler: F) {
        self.connect_local("notify::playing", true, move |values| {
            if let Ok(_self) = values[0].get::<Self>() {
                handler(&_self);
            }
            None
        })
        .expect("connecting to prop 'playing' failed");
    }

    pub fn connect_selected_local<F: Fn(&Self) + 'static>(&self, handler: F) {
        self.connect_local("notify::selected", true, move |values| {
            if let Ok(_self) = values[0].get::<Self>() {
                handler(&_self);
            }
            None
        })
        .expect("connecting to prop 'selected' failed");
    }
}

mod imp {

    use super::*;
    use std::cell::RefCell;

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
    #[glib::object_subclass]
    impl ObjectSubclass for SongModel {
        // This type name must be unique per process.
        const NAME: &'static str = "SongModel";

        type Type = super::SongModel;

        // The parent type this one is inheriting from.
        type ParentType = glib::Object;

        // Interfaces this type implements
        type Interfaces = ();

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

    // Static array for defining the properties of the new type.
    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 7] = [
            glib::ParamSpec::new_uint(
                "index",
                "Index",
                "",
                1,
                u32::MAX,
                1,
                glib::ParamFlags::READWRITE,
            ),
            glib::ParamSpec::new_string("title", "Title", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string("artist", "Artist", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string("id", "id", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string(
                "duration",
                "Duration",
                "",
                None,
                glib::ParamFlags::READWRITE,
            ),
            glib::ParamSpec::new_boolean(
                "playing",
                "Playing",
                "",
                false,
                glib::ParamFlags::READWRITE
            ),
            glib::ParamSpec::new_boolean(
                "selected",
                "Selected",
                "",
                false,
                glib::ParamFlags::READWRITE,
            ),
        ];
    }

    // Trait that is used to override virtual methods of glib::Object.
    impl ObjectImpl for SongModel {
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
                "index" => {
                    let index = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.index.replace(index);
                }
                "title" => {
                    let title = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.title.replace(title);
                }
                "artist" => {
                    let artist = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.artist.replace(artist);
                }
                "id" => {
                    let id = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.id.replace(id);
                }
                "duration" => {
                    let dur = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.duration.replace(dur);
                }
                "playing" => {
                    let playing = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.playing.replace(playing);
                }
                "selected" => {
                    let selected = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.selected.replace(selected);
                }
                _ => unimplemented!(),
            }
        }

        // Called whenever a property is retrieved from this instance. The id
        // is the same as the index of the property in the PROPERTIES array.
        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "index" => self.index.borrow().to_value(),
                "title" => self.title.borrow().to_value(),
                "artist" => self.artist.borrow().to_value(),
                "id" => self.id.borrow().to_value(),
                "duration" => self.duration.borrow().to_value(),
                "playing" => self.playing.borrow().to_value(),
                "selected" => self.selected.borrow().to_value(),
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
