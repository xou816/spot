use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use super::SidebarItem;

impl SidebarRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of SidebarRow")
    }

    pub fn set_item(&self, item: &SidebarItem) {
        self.set_property("item", item);
        self.imp().title.set_text(item.title().as_str());
        self.imp().icon.set_icon_name(item.icon());
        self.set_tooltip_text(Option::from(item.title().as_str()));
    }

    pub fn item(&self) -> SidebarItem {
        self.property("item")
    }
}

mod imp {
    use super::*;
    use gdk::cairo::glib::ParamSpec;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/sidebar/sidebar_row.ui")]
    pub struct SidebarRow {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,

        #[template_child]
        pub title: TemplateChild<gtk::Label>,

        pub item: RefCell<Option<SidebarItem>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarRow {
        const NAME: &'static str = "SidebarRow";
        type Type = super::SidebarRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 1] = [glib::ParamSpecObject::new(
            "item",
            "Item",
            "",
            SidebarItem::static_type(),
            glib::ParamFlags::READWRITE
        ),];
    }

    impl ObjectImpl for SidebarRow {
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
                "item" => {
                    let item: Option<SidebarItem> = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.item.replace(item);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => self.item.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SidebarRow {}

    impl ListBoxRowImpl for SidebarRow {}
}

glib::wrapper! {
    pub struct SidebarRow(ObjectSubclass<imp::SidebarRow>) @extends gtk::Widget, gtk::ListBoxRow;
}
