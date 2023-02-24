use crate::app::components::display_add_css_provider;
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::models::AlbumModel;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use libadwaita::subclass::prelude::BinImpl;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/album.ui")]
    pub struct AlbumWidget {
        #[template_child]
        pub album_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub artist_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub year_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub cover_btn: TemplateChild<gtk::Button>,

        #[template_child]
        pub cover_image: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumWidget {
        const NAME: &'static str = "AlbumWidget";
        type Type = super::AlbumWidget;
        type ParentType = libadwaita::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AlbumWidget {}
    impl WidgetImpl for AlbumWidget {}
    impl BinImpl for AlbumWidget {}
}

glib::wrapper! {
    pub struct AlbumWidget(ObjectSubclass<imp::AlbumWidget>) @extends gtk::Widget, libadwaita::Bin;
}

impl AlbumWidget {
    pub fn new() -> Self {
        display_add_css_provider(resource!("/components/album.css"));
        glib::Object::new()
    }

    pub fn for_model(album_model: &AlbumModel, worker: Worker) -> Self {
        let _self = Self::new();
        _self.bind(album_model, worker);
        _self
    }

    fn set_loaded(&self) {
        let context = self.style_context();
        context.add_class("container--loaded");
    }

    fn set_image(&self, pixbuf: Option<&gdk_pixbuf::Pixbuf>) {
        self.imp().cover_image.set_from_pixbuf(pixbuf);
    }

    fn bind(&self, album_model: &AlbumModel, worker: Worker) {
        let widget = self.imp();
        widget.cover_image.set_overflow(gtk::Overflow::Hidden);

        if let Some(cover_art) = album_model.cover() {
            let _self = self.downgrade();
            worker.send_local_task(async move {
                if let Some(_self) = _self.upgrade() {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&cover_art, "jpg", 200, 200).await;
                    _self.set_image(result.as_ref());
                    _self.set_loaded();
                }
            });
        } else {
            self.set_loaded();
        }

        album_model
            .bind_property("album", &*widget.album_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        album_model
            .bind_property("artist", &*widget.artist_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        if album_model.year() > 0 {
            album_model
                .bind_property("year", &*widget.year_label, "label")
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build();
        } else {
            widget.year_label.hide();
        }
    }

    pub fn connect_album_pressed<F: Fn(&Self) + 'static>(&self, f: F) {
        self.imp()
            .cover_btn
            .connect_clicked(clone!(@weak self as _self => move |_| {
                f(&_self);
            }));
    }
}
