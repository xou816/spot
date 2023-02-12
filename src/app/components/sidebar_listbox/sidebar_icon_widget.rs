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
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SideBarItemWidgetIcon {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
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
        let s: Self = glib::Object::new();
        s.imp().title.set_text(title);
        s.imp().icon.set_icon_name(iconname);
        s.set_tooltip_text(Option::from(title));
        s
    }
}
