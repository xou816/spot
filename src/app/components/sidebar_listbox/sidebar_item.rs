use gtk::prelude::*;
use gtk::subclass::prelude::*;

impl SideBarItem {
    pub fn new(id: &str, title: &str, iconname: &str, grayedout: bool) -> SideBarItem {
        glib::Object::new::<SideBarItem>(&[
            ("id", &id),
            ("title", &title),
            ("iconname", &iconname),
            ("grayedout", &grayedout),
        ])
        .expect("Failed to create")
    }
}

mod imp {
    use super::*;
    use gdk::cairo::glib::ParamSpec;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct SideBarItem {
        pub id: RefCell<Option<String>>,
        pub title: RefCell<Option<String>>,
        pub iconname: RefCell<Option<String>>,
        pub grayedout: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SideBarItem {
        const NAME: &'static str = "SideBarItem";
        type Type = super::SideBarItem;
        type ParentType = glib::Object;

        fn new() -> Self {
            Self {
                id: RefCell::new(None),
                title: RefCell::new(None),
                iconname: RefCell::new(None),
                grayedout: RefCell::new(false),
            }
        }
    }

    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 4] = [
            glib::ParamSpec::new_string("id", "ID", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string("title", "Title", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpec::new_string(
                "iconname",
                "IconName",
                "",
                None,
                glib::ParamFlags::READWRITE
            ),
            glib::ParamSpec::new_boolean(
                "grayedout",
                "GrayedOut",
                "",
                false,
                glib::ParamFlags::READWRITE
            ),
        ];
    }

    impl ObjectImpl for SideBarItem {
        fn properties() -> &'static [ParamSpec] {
            &*PROPERTIES
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "id" => {
                    let id = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.id.replace(id);
                }
                "title" => {
                    let title = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.title.replace(title);
                }
                "iconname" => {
                    let iconname = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.iconname.replace(iconname);
                }
                "grayedout" => {
                    let grayedout = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.grayedout.replace(grayedout);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.borrow().to_value(),
                "title" => self.title.borrow().to_value(),
                "iconname" => self.iconname.borrow().to_value(),
                "grayedout" => self.grayedout.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SideBarItem(ObjectSubclass<imp::SideBarItem>);
}
