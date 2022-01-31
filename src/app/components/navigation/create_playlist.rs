use crate::app::components::HomeModel;
use glib::SignalHandlerId;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/create_playlist.ui")]
    pub struct CreatePlaylistWidget {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CreatePlaylistWidget {
        const NAME: &'static str = "CreatePlaylistWidget";
        type Type = super::CreatePlaylistWidget;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CreatePlaylistWidget {}
    impl WidgetImpl for CreatePlaylistWidget {}
    impl PopoverImpl for CreatePlaylistWidget {}
}

glib::wrapper! {
    pub struct CreatePlaylistWidget(ObjectSubclass<imp::CreatePlaylistWidget>) @extends gtk::Widget, gtk::Popover;
}

impl CreatePlaylistWidget {
    pub fn new(parent: &gtk::ListBoxRow, model: Rc<HomeModel>, user: Rc<String>) -> Self {
        let w: CreatePlaylistWidget =
            glib::Object::new(&[]).expect("Failed to create an instance of CreatePlaylistWidget");
        w.connect_button_clicked(model, user);
        w.set_parent(parent);
        w.popup();
        w
    }

    pub fn connect_button_clicked(
        &self,
        model: Rc<HomeModel>,
        user: Rc<String>,
    ) -> SignalHandlerId {
        let widget = imp::CreatePlaylistWidget::from_instance(self);
        let btn = widget.button.get();
        let entry = widget.entry.get();
        btn.connect_clicked(clone!(@weak self as _self => move |_| {
            model.create_new_playlist(entry.text().to_string(), user.to_string());
            _self.popdown();
        }))
    }
}
