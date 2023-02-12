use crate::app::components::display_add_css_provider;
use crate::app::components::sidebar_listbox::sidebar_icon_widget::SideBarItemWidgetIcon;
use crate::app::components::sidebar_listbox::SideBarItem;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

impl SideBarRow {
    pub fn new(item: &SideBarItem) -> SideBarRow {
        display_add_css_provider(resource!("/sidebar_listbox/sidebar.css"));
        glib::Object::builder::<Self>()
            .property("item", item)
            .build()
    }

    pub fn id(&self) -> String {
        self.item().id()
    }
}

mod imp {
    use super::*;
    use gdk::cairo::glib::ParamSpec;
    use glib::Properties;
    use std::cell::RefCell;
    use std::convert::TryFrom;

    #[derive(Debug, Properties)]
    #[properties(wrapper_type = super::SideBarRow)]
    pub struct SideBarRow {
        #[property(get, set = Self::set_item)]
        pub item: RefCell<SideBarItem>,
    }

    impl SideBarRow {
        fn set_item(&self, item: SideBarItem) {
            let t = item.title();
            let row = self.obj();

            if item.grayed_out() {
                row.set_activatable(false);
                row.set_selectable(false);
                row.set_sensitive(false);

                let label = gtk::Label::new(Some(t.as_str()));
                label.add_css_class("caption-heading");
                label.add_css_class("item_sidebar");

                let _box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                _box.append(&label);

                row.set_child(Some(&_box));
            } else {
                let child = SideBarItemWidgetIcon::new(t.as_str(), Some(item.icon_name().as_str()));
                row.set_child(Some(&child));
            }

            self.item.replace(item);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SideBarRow {
        const NAME: &'static str = "SideBarRow";
        type Type = super::SideBarRow;
        type ParentType = gtk::ListBoxRow;

        fn new() -> Self {
            Self {
                item: RefCell::new(glib::Object::new()),
            }
        }
    }

    impl ObjectImpl for SideBarRow {
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

    impl WidgetImpl for SideBarRow {}
    impl ListBoxRowImpl for SideBarRow {}
}

glib::wrapper! {
    pub struct SideBarRow(ObjectSubclass<imp::SideBarRow>) @extends gtk::Widget, gtk::ListBoxRow;
}
