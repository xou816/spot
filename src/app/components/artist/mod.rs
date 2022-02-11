use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::app::loader::ImageLoader;
use crate::app::models::ArtistModel;
use crate::app::Worker;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/artist.ui")]
    pub struct ArtistWidget {
        #[template_child]
        pub artist: TemplateChild<gtk::Label>,

        #[template_child]
        pub avatar_btn: TemplateChild<gtk::Button>,

        #[template_child]
        pub avatar: TemplateChild<libadwaita::Avatar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArtistWidget {
        const NAME: &'static str = "ArtistWidget";
        type Type = super::ArtistWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArtistWidget {}
    impl WidgetImpl for ArtistWidget {}
    impl BoxImpl for ArtistWidget {}
}

glib::wrapper! {
    pub struct ArtistWidget(ObjectSubclass<imp::ArtistWidget>) @extends gtk::Widget, gtk::Box;
}

impl ArtistWidget {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of ArtistWidget")
    }

    pub fn for_model(model: &ArtistModel, worker: Worker) -> Self {
        let _self = Self::new();
        _self.bind(model, worker);
        _self
    }

    pub fn connect_artist_pressed<F: Fn(&Self) + 'static>(&self, f: F) {
        imp::ArtistWidget::from_instance(self)
            .avatar_btn
            .connect_clicked(clone!(@weak self as _self => move |_| {
                f(&_self);
            }));
    }

    fn bind(&self, model: &ArtistModel, worker: Worker) {
        let widget = imp::ArtistWidget::from_instance(self);

        if let Some(url) = model.image_url() {
            let avatar = widget.avatar.downgrade();
            worker.send_local_task(async move {
                if let Some(avatar) = avatar.upgrade() {
                    let loader = ImageLoader::new();
                    let pixbuf = loader.load_remote(&url, "jpg", 200, 200).await;
                    let texture = pixbuf.as_ref().map(gdk::Texture::for_pixbuf);
                    avatar.set_custom_image(texture.as_ref());
                }
            });
        }

        model
            .bind_property("artist", &*widget.artist, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
    }
}
