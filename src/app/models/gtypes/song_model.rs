#![allow(clippy::all)]

use gio::prelude::*;
use glib::{subclass::prelude::*, SignalHandlerId};
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

    pub fn bind_index(&self, o: &impl ObjectType, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("index", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_artist(&self, o: &impl ObjectType, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("artist", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_title(&self, o: &impl ObjectType, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("title", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_duration(&self, o: &impl ObjectType, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("duration", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_playing(&self, o: &impl ObjectType, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("playing", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_selected(&self, o: &impl ObjectType, property: &str) {
        imp::SongModel::from_instance(self).push_binding(
            self.bind_property("selected", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn unbind_all(&self) {
        imp::SongModel::from_instance(self).unbind_all(self);
    }

    pub fn description(&self) -> impl Deref<Target = SongDescription> + '_ {
        Ref::map(imp::SongModel::from_instance(self).song.borrow(), |s| {
            s.as_ref().expect("song set at constructor")
        })
    }

    pub fn into_description(&self) -> SongDescription {
        imp::SongModel::from_instance(&self)
            .song
            .borrow()
            .as_ref()
            .cloned()
            .expect("song set at constructor")
    }
}

#[derive(Copy, Clone, Default)]
pub struct SongState {
    pub is_playing: bool,
    pub is_selected: bool,
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
        pub state: Cell<SongState>,
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
                    let is_playing = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    let SongState { is_selected, .. } = self.state.get();
                    self.state.set(SongState {
                        is_playing,
                        is_selected,
                    });
                }
                "selected" => {
                    let is_selected = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    let SongState { is_playing, .. } = self.state.get();
                    self.state.set(SongState {
                        is_playing,
                        is_selected,
                    });
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
                "playing" => self.state.get().is_playing.to_value(),
                "selected" => self.state.get().is_selected.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
