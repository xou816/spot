#![allow(clippy::all)]

use gio::prelude::*;
use glib::{subclass::prelude::*, SignalHandlerId};
use std::{cell::Ref, ops::Deref};

use crate::app::components::utils::format_duration;
use crate::app::models::*;

glib::wrapper! {
    pub struct SongModel(ObjectSubclass<imp::SongModel>);
}

impl SongModel {
    pub fn new(song: SongDescription) -> Self {
        let o: Self = glib::Object::new();
        o.imp().song.replace(Some(song));
        o
    }

    pub fn set_playing(&self, is_playing: bool) {
        self.set_property("playing", is_playing);
    }

    pub fn set_selected(&self, is_selected: bool) {
        self.set_property("selected", is_selected);
    }

    pub fn get_playing(&self) -> bool {
        self.property("playing")
    }

    pub fn get_selected(&self) -> bool {
        self.property("selected")
    }

    pub fn get_id(&self) -> String {
        self.property("id")
    }

    pub fn bind_index(&self, o: &impl ObjectType, property: &str) {
        self.imp().push_binding(
            self.bind_property("index", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_artist(&self, o: &impl ObjectType, property: &str) {
        self.imp().push_binding(
            self.bind_property("artist", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_title(&self, o: &impl ObjectType, property: &str) {
        self.imp().push_binding(
            self.bind_property("title", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_duration(&self, o: &impl ObjectType, property: &str) {
        self.imp().push_binding(
            self.bind_property("duration", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_playing(&self, o: &impl ObjectType, property: &str) {
        self.imp().push_binding(
            self.bind_property("playing", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn bind_selected(&self, o: &impl ObjectType, property: &str) {
        self.imp().push_binding(
            self.bind_property("selected", o, property)
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build(),
        );
    }

    pub fn unbind_all(&self) {
        self.imp().unbind_all(self);
    }

    pub fn description(&self) -> impl Deref<Target = SongDescription> + '_ {
        Ref::map(self.imp().song.borrow(), |s| {
            s.as_ref().expect("song set at constructor")
        })
    }

    pub fn into_description(&self) -> SongDescription {
        self.imp()
            .song
            .borrow()
            .as_ref()
            .cloned()
            .expect("song set at constructor")
    }
}

mod imp {

    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    struct BindingsInner {
        pub signals: Vec<SignalHandlerId>,
        pub bindings: Vec<glib::Binding>,
    }

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

        pub fn push_binding(&self, binding: glib::Binding) {
            self.bindings.borrow_mut().bindings.push(binding);
        }

        pub fn unbind_all<O: ObjectExt>(&self, o: &O) {
            let mut bindings = self.bindings.borrow_mut();
            bindings.signals.drain(..).for_each(|s| o.disconnect(s));
            bindings.bindings.drain(..).for_each(|b| b.unbind());
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongModel {
        const NAME: &'static str = "SongModel";
        type Type = super::SongModel;
        type ParentType = glib::Object;
    }

    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 8] = [
            glib::ParamSpecString::builder("id").read_only().build(),
            glib::ParamSpecUInt::builder("index").read_only().build(),
            glib::ParamSpecString::builder("title").read_only().build(),
            glib::ParamSpecString::builder("artist").read_only().build(),
            glib::ParamSpecString::builder("duration")
                .read_only()
                .build(),
            glib::ParamSpecString::builder("art").read_only().build(),
            glib::ParamSpecBoolean::builder("playing")
                .readwrite()
                .build(),
            glib::ParamSpecBoolean::builder("selected")
                .readwrite()
                .build(),
        ];
    }

    impl ObjectImpl for SongModel {
        fn properties() -> &'static [glib::ParamSpec] {
            &*PROPERTIES
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
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

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
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
