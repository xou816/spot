use glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

impl SideBarItem {
    pub fn new(id: &str, title: &str, icon_name: &str, grayed_out: bool) -> SideBarItem {
        glib::Object::builder()
            .property("id", &id)
            .property("title", &title)
            .property("icon-name", &icon_name)
            .property("grayed-out", &grayed_out)
            .build()
    }
}

mod imp {
    use super::*;
    use gdk::cairo::glib::ParamSpec;
    use std::cell::{Cell, RefCell};
    use std::convert::TryFrom;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SideBarItem)]
    pub struct SideBarItem {
        #[property(get, set)]
        pub id: RefCell<String>,

        #[property(get, set)]
        pub title: RefCell<String>,

        #[property(get, set, name = "icon-name")]
        pub icon_name: RefCell<String>,

        #[property(get, set, name = "grayed-out")]
        pub grayed_out: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SideBarItem {
        const NAME: &'static str = "SideBarItem";
        type Type = super::SideBarItem;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for SideBarItem {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct SideBarItem(ObjectSubclass<imp::SideBarItem>);
}
