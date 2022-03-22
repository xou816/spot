use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/sidebar_listbox/sidebar_icon_widget.ui")]
    pub struct SideBarItemWidgetIcon {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,

        #[template_child]
        pub title: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SideBarItemWidgetIcon {
        const NAME: &'static str = "SideBarItemWidgetIcon";
        type Type = super::SideBarItemWidgetIcon;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SideBarItemWidgetIcon {
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for SideBarItemWidgetIcon {}
    impl BoxImpl for SideBarItemWidgetIcon {}
}

glib::wrapper! {
    pub struct SideBarItemWidgetIcon(ObjectSubclass<imp::SideBarItemWidgetIcon>) @extends gtk::Widget, gtk::Box;
}

impl SideBarItemWidgetIcon {
    pub fn new(title: &str, iconname: Option<&str>) -> Self {
        let s: SideBarItemWidgetIcon =
            glib::Object::new(&[]).expect("Failed to create an instance of SideBarItemWidgetIcon");
        s.widget().title.set_text(title);
        s.widget().icon.set_icon_name(iconname);
        s.set_tooltip_text(Option::from(title));
        s
    }

    pub fn widget(&self) -> &imp::SideBarItemWidgetIcon {
        imp::SideBarItemWidgetIcon::from_instance(self)
    }
}
