#![allow(clippy::all)]

use gio::prelude::*;
use glib::subclass::prelude::*;
use glib::Properties;

// UI model!
glib::wrapper! {
    pub struct ArtistModel(ObjectSubclass<imp::ArtistModel>);
}

impl ArtistModel {
    pub fn new(artist: &str, image: &Option<String>, id: &str) -> ArtistModel {
        glib::Object::builder()
            .property("artist", &artist)
            .property("image", image)
            .property("id", &id)
            .build()
    }
}

mod imp {

    use super::*;
    use std::cell::RefCell;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ArtistModel)]
    pub struct ArtistModel {
        #[property(get, set)]
        artist: RefCell<String>,
        #[property(get, set)]
        image: RefCell<Option<String>>,
        #[property(get, set)]
        id: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArtistModel {
        const NAME: &'static str = "ArtistModel";
        type Type = super::ArtistModel;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ArtistModel {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}
