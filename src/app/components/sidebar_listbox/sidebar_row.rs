use gtk::prelude::*;
use gtk::subclass::prelude::*;

impl SideBarRow {
    pub fn new(id: &str) -> SideBarRow {
        glib::Object::new::<SideBarRow>(&[("id", &id)]).expect("Failed to create")
    }
}

mod imp {
    use super::*;
    use gdk::cairo::glib::ParamSpec;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct SideBarRow {
        pub id: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SideBarRow {
        const NAME: &'static str = "SideBarRow";
        type Type = super::SideBarRow;
        type ParentType = gtk::ListBoxRow;

        fn new() -> Self {
            Self {
                id: RefCell::new(None),
            }
        }
    }

    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 1] = [glib::ParamSpec::new_string(
            "id",
            "ID",
            "",
            None,
            glib::ParamFlags::READWRITE
        ),];
    }

    impl ObjectImpl for SideBarRow {
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SideBarRow {}

    impl ListBoxRowImpl for SideBarRow {}
}
glib::wrapper! {
    pub struct SideBarRow(ObjectSubclass<imp::SideBarRow>) @extends gtk::Widget, gtk::ListBoxRow;
}
