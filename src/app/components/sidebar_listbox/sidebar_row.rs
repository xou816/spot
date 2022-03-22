use crate::app::components::display_add_css_provider;
use crate::app::components::sidebar_listbox::sidebar_icon_widget::SideBarItemWidgetIcon;
use crate::app::components::sidebar_listbox::SideBarItem;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

impl SideBarRow {
    pub fn new(item: &SideBarItem) -> SideBarRow {
        display_add_css_provider(resource!("/sidebar_listbox/sidebar.css"));
        let id = item.id();
        let t = item.title();
        let row =
            glib::Object::new::<SideBarRow>(&[("id", &id.as_str())]).expect("Failed to create");
        if item.grayedout() {
            let label = gtk::Label::new(Option::from(t.as_str()));
            let gtk_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            gtk_box.append(&label);
            row.set_child(Option::from(&gtk_box));
            row.set_activatable(false);
            row.set_selectable(false);
            row.set_sensitive(false);
            label.add_css_class("caption-heading");
            label.add_css_class("item_sidebar");
        } else {
            let widget = SideBarItemWidgetIcon::new(t.as_str(), Some(item.iconname().as_str()));
            row.set_child(Option::from(&widget));
        }
        row
    }

    pub fn id(&self) -> String {
        self.property::<String>("id")
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
        static ref PROPERTIES: [glib::ParamSpec; 1] = [glib::ParamSpecString::new(
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
