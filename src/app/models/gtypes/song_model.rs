#![allow(clippy::all)]

use gio::prelude::*;
use glib::{subclass::prelude::*, SignalHandlerId};
use ref_filter_map::ref_filter_map;
use std::{cell::Ref, ops::Deref};

use crate::app::models::SongDescription;
glib::wrapper! {
    pub struct SongModel(ObjectSubclass<imp::SongModel>);
}

// Constructor for new instances. This simply calls glib::Object::new() with
// initial values for our two properties and then returns the new instance
impl SongModel {
    pub fn new(song: SongDescription) -> Self {
        let o: Self = glib::Object::new(&[]).unwrap();
        imp::SongModel::from_instance(&o).song.replace(Some(song));
        o
    }

    pub fn cover_url(&self) -> Option<String> {
        self.property("art")
            .unwrap()
            .get::<&str>()
            .ok()
            .map(|s| s.to_string())
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

    pub fn bind_index<O: ObjectType>(&self, o: &O, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("index", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_artist<O: ObjectType>(&self, o: &O, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("artist", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_title<O: ObjectType>(&self, o: &O, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("title", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_duration<O: ObjectType>(&self, o: &O, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("duration", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn connect_playing_local<F: Fn(&Self) + 'static>(&self, handler: F) {
        let signal_id = self
            .connect_local("notify::playing", true, move |values| {
                if let Ok(_self) = values[0].get::<Self>() {
                    handler(&_self);
                }
                None
            })
            .expect("connecting to prop 'playing' failed");
        imp::SongModel::from_instance(self).push_signal(signal_id);
    }

    pub fn connect_selected_local<F: Fn(&Self) + 'static>(&self, handler: F) {
        let signal_id = self
            .connect_local("notify::selected", true, move |values| {
                if let Ok(_self) = values[0].get::<Self>() {
                    handler(&_self);
                }
                None
            })
            .expect("connecting to prop 'selected' failed");
        imp::SongModel::from_instance(self).push_signal(signal_id);
    }

    pub fn unbind_all(&self) {
        imp::SongModel::from_instance(self).unbind_all(self);
    }

    pub fn as_song_description(&self) -> impl Deref<Target = SongDescription> + '_ {
        Ref::map(imp::SongModel::from_instance(self).song.borrow(), |s| {
            s.as_ref().unwrap()
        })
    }
}

mod imp {

    use crate::app::components::utils::format_duration;

    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    struct BindingsInner {
        pub signals: Vec<SignalHandlerId>,
        pub bindings: Vec<glib::Binding>,
    }

    // This is the struct containing all state carried with
    // the new type. Generally this has to make use of
    // interior mutability.
    #[derive(Default)]
    pub struct SongModel {
        pub song: RefCell<Option<SongDescription>>,
        playing: Cell<bool>,
        selected: Cell<bool>,
        bindings: RefCell<BindingsInner>,
    }

    impl SongModel {
        pub fn push_signal(&self, id: SignalHandlerId) {
            self.bindings.borrow_mut().signals.push(id);
        }

        pub fn push_binding(&self, binding: Option<glib::Binding>) {
            if let Some(binding) = binding {
                self.bindings.borrow_mut().bindings.push(binding);
            }
        }

        pub fn unbind_all<O: ObjectExt>(&self, o: &O) {
            let mut bindings = self.bindings.borrow_mut();
            bindings.signals.drain(..).for_each(|s| o.disconnect(s));
            bindings.bindings.drain(..).for_each(|b| b.unbind());
        }
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
    }

    // Static array for defining the properties of the new type.
    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 8] = [
            glib::ParamSpec::new_string(
                "id",
                "Spotify identifier",
                "",
                None,
                glib::ParamFlags::READABLE
            ),
            glib::ParamSpec::new_uint(
                "index",
                "Track number within an album",
                "",
                1,
                u32::MAX,
                1,
                glib::ParamFlags::READABLE,
            ),
            glib::ParamSpec::new_string(
                "title",
                "Title of the track",
                "",
                None,
                glib::ParamFlags::READABLE
            ),
            glib::ParamSpec::new_string(
                "artist",
                "Artists, comma separated",
                "",
                None,
                glib::ParamFlags::READABLE
            ),
            glib::ParamSpec::new_string(
                "duration",
                "Duration (formatted)",
                "",
                None,
                glib::ParamFlags::READABLE,
            ),
            glib::ParamSpec::new_string(
                "art",
                "URL to the cover art",
                "",
                None,
                glib::ParamFlags::READABLE,
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
                "index" => self
                    .song
                    .borrow()
                    .as_ref()
                    .expect("song set at constructor")
                    .track_number
                    .unwrap_or(1)
                    .to_value(),
                "title" => self
                    .song
                    .borrow()
                    .as_ref()
                    .expect("song set at constructor")
                    .title
                    .to_value(),
                "artist" => self
                    .song
                    .borrow()
                    .as_ref()
                    .expect("song set at constructor")
                    .artists_name()
                    .to_value(),
                "id" => self
                    .song
                    .borrow()
                    .as_ref()
                    .expect("song set at constructor")
                    .id
                    .to_value(),
                "duration" => self
                    .song
                    .borrow()
                    .as_ref()
                    .map(|s| format_duration(s.duration.into()))
                    .expect("song set at constructor")
                    .to_value(),
                "art" => self
                    .song
                    .borrow()
                    .as_ref()
                    .expect("song set at constructor")
                    .art
                    .to_value(),
                "playing" => self.playing.get().to_value(),
                "selected" => self.selected.get().to_value(),
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
