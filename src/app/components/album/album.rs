use crate::app::components::display_add_css_provider;
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::models::AlbumModel;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use libadwaita::subclass::prelude::BinImpl;
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/album.ui")]
    pub struct AlbumWidget {
        pub clicked_sig_handle: RefCell<Option<glib::SignalHandlerId>>,

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
            Self::bind_template(klass);
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
        glib::Object::new(&[]).expect("Failed to create an instance of AlbumWidget")
    }

    /// Binds a ListItem widget to an item in a model.
    ///
    /// Has to be unbound due to connected signals,
    pub fn bind_list_item<F>(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem, worker: Worker, f: F)
    where
        F: Fn(String) + 'static
    {
        let album_model = list_item.item().unwrap().downcast::<AlbumModel>().unwrap();

        let imp = list_item.child().unwrap().downcast::<AlbumWidget>().unwrap();
        let widget = imp::AlbumWidget::from_instance(&imp);

        // TODO: Get rid of this (maybe XML?)
        widget.cover_image.set_overflow(gtk::Overflow::Hidden);

        if let Some(url) = album_model.cover_url() {
            let _imp = imp.downgrade();
            worker.send_local_task(async move {
                if let Some(_imp) = _imp.upgrade() {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 200, 200).await;
                    _imp.set_image(result.as_ref());
                    _imp.set_loaded();
                }
            });
        } else {
            imp.set_loaded();
        }

        album_model
            .bind_property("album", &*widget.album_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        album_model
            .bind_property("artist", &*widget.artist_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        match album_model.year() {
            Some(_) => {
                album_model
                    .bind_property("year", &*widget.year_label, "label")
                    .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                    .build();
            }
            None => {
                widget.year_label.hide();
            }
        }

        widget.clicked_sig_handle.replace(Some(
            widget.cover_btn.connect_clicked(clone!(@weak album_model => move |_| {
                f(album_model.uri())
            }))
        ));
    }

    /// Unbinds a ListItem widget from an item in a model
    pub fn unbind_list_item(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
        let imp = list_item.child().unwrap().downcast::<AlbumWidget>().unwrap();
        let widget = imp::AlbumWidget::from_instance(&imp);
        let sig_handle = widget.clicked_sig_handle.take().unwrap();

        glib::Object::disconnect(
            &imp.upcast::<glib::Object>(),
            sig_handle
        );
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
        imp::AlbumWidget::from_instance(self)
            .cover_image
            .set_from_pixbuf(pixbuf);
    }

    fn bind(&self, album_model: &AlbumModel, worker: Worker) {
        let widget = imp::AlbumWidget::from_instance(self);
        widget.cover_image.set_overflow(gtk::Overflow::Hidden);

        if let Some(url) = album_model.cover_url() {
            let _self = self.downgrade();
            worker.send_local_task(async move {
                if let Some(_self) = _self.upgrade() {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 200, 200).await;
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

        match album_model.year() {
            Some(_) => {
                album_model
                    .bind_property("year", &*widget.year_label, "label")
                    .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                    .build();
            }
            None => {
                widget.year_label.hide();
            }
        }
    }

    pub fn connect_album_pressed<F: Fn(&Self) + 'static>(&self, f: F) {
        imp::AlbumWidget::from_instance(self)
            .cover_btn
            .connect_clicked(clone!(@weak self as _self => move |_| {
                f(&_self);
            }));
    }
}
