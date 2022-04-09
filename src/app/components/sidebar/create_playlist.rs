use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/create_playlist.ui")]
    pub struct CreatePlaylistPopover {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CreatePlaylistPopover {
        const NAME: &'static str = "CreatePlaylistPopover";
        type Type = super::CreatePlaylistPopover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CreatePlaylistPopover {}
    impl WidgetImpl for CreatePlaylistPopover {}
    impl PopoverImpl for CreatePlaylistPopover {}
}

glib::wrapper! {
    pub struct CreatePlaylistPopover(ObjectSubclass<imp::CreatePlaylistPopover>) @extends gtk::Widget, gtk::Popover;
}

impl CreatePlaylistPopover {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of CreatePlaylistPopover")
    }

    pub fn connect_create<F: Clone + Fn(String) + 'static>(&self, create_fun: F) {
        let entry = self.imp().entry.get();
        let closure = clone!(@weak self as popover, @weak entry, @strong create_fun => move || {
            create_fun(entry.text().to_string());
            popover.popdown();
            entry.buffer().delete_text(0, None);
        });
        let closure_clone = closure.clone();
        entry.connect_activate(move |_| closure());
        self.imp().button.connect_clicked(move |_| closure_clone());
    }
}
