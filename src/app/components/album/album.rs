use crate::app::components::screen_add_css_provider;
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::models::AlbumModel;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/album.ui")]
    pub struct AlbumWidget {
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub album_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub artist_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub cover_btn: TemplateChild<gtk::Button>,

        #[template_child]
        pub cover_image: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumWidget {
        const NAME: &'static str = "AlbumWidget";
        type Type = super::AlbumWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AlbumWidget {}
    impl WidgetImpl for AlbumWidget {}
    impl BoxImpl for AlbumWidget {}
}

glib::wrapper! {
    pub struct AlbumWidget(ObjectSubclass<imp::AlbumWidget>) @extends gtk::Widget, gtk::Box;
}

impl AlbumWidget {
    pub fn new() -> Self {
        screen_add_css_provider(resource!("/components/album.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of AlbumWidget")
    }

    pub fn for_model(album_model: &AlbumModel, worker: Worker) -> Self {
        let _self = Self::new();
        _self.bind(album_model, worker);
        _self
    }

    fn bind(&self, album_model: &AlbumModel, worker: Worker) {
        let widget = imp::AlbumWidget::from_instance(self);

        let image = widget.cover_image.downgrade();
        let revealer = widget.revealer.downgrade();
        if let Some(url) = album_model.cover_url() {
            worker.send_local_task(async move {
                if let (Some(image), Some(revealer)) = (image.upgrade(), revealer.upgrade()) {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 200, 200).await;
                    image.set_from_pixbuf(result.as_ref());
                    revealer.set_reveal_child(true);
                }
            });
        } else {
            widget.revealer.set_reveal_child(true);
        }

        album_model
            .bind_property("album", &*widget.album_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        album_model
            .bind_property("artist", &*widget.artist_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
    }

    pub fn connect_album_pressed<F: Fn(&Self) + 'static>(&self, f: F) {
        imp::AlbumWidget::from_instance(self)
            .cover_btn
            .connect_clicked(clone!(@weak self as _self => move |_| {
                f(&_self);
            }));
    }
}
