use crate::app::components::display_add_css_provider;
use crate::app::loader::ImageLoader;
use crate::app::models::SongModel;

use crate::app::Worker;
use gio::MenuModel;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/song.ui")]
    pub struct SongWidget {
        #[template_child]
        pub song_index: TemplateChild<gtk::Label>,

        #[template_child]
        pub song_icon: TemplateChild<gtk::Spinner>,

        #[template_child]
        pub song_checkbox: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub song_title: TemplateChild<gtk::Label>,

        #[template_child]
        pub song_artist: TemplateChild<gtk::Label>,

        #[template_child]
        pub song_length: TemplateChild<gtk::Label>,

        #[template_child]
        pub menu_btn: TemplateChild<gtk::MenuButton>,

        #[template_child]
        pub song_cover: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongWidget {
        const NAME: &'static str = "SongWidget";
        type Type = super::SongWidget;
        type ParentType = gtk::Grid;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SongWidget {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.song_checkbox.set_sensitive(false);
        }
    }

    impl WidgetImpl for SongWidget {}
    impl GridImpl for SongWidget {}
}

glib::wrapper! {
    pub struct SongWidget(ObjectSubclass<imp::SongWidget>) @extends gtk::Widget, gtk::Grid;
}

impl SongWidget {
    pub fn new() -> Self {
        display_add_css_provider(resource!("/components/song.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of SongWidget")
    }

    pub fn for_model(model: SongModel, worker: Worker) -> Self {
        let _self = Self::new();
        _self.bind(&model, worker, true);
        _self
    }

    pub fn set_actions(&self, actions: Option<&gio::ActionGroup>) {
        self.insert_action_group("song", actions);
    }

    pub fn set_menu(&self, menu: Option<&MenuModel>) {
        if menu.is_some() {
            let widget = imp::SongWidget::from_instance(self);
            widget.menu_btn.set_menu_model(menu);
            widget
                .menu_btn
                .style_context()
                .add_class("song__menu--enabled");
        }
    }

    fn set_playing(&self, is_playing: bool) {
        let song_class = "song--playing";
        let context = self.style_context();
        if is_playing {
            context.add_class(song_class);
        } else {
            context.remove_class(song_class);
        }
    }

    fn set_selected(&self, is_selected: bool) {
        imp::SongWidget::from_instance(self)
            .song_checkbox
            .set_active(is_selected);
        let song_class = "song-selected";
        let context = self.style_context();
        if is_selected {
            context.add_class(song_class);
        } else {
            context.remove_class(song_class);
        }
    }

    fn set_image(&self, pixbuf: Option<&gdk_pixbuf::Pixbuf>) {
        imp::SongWidget::from_instance(self)
            .song_cover
            .set_from_pixbuf(pixbuf);
    }

    pub fn bind_art(&self, model: &SongModel, worker: Worker) {
        if let Some(url) = model.cover_url() {
            let _self = self.downgrade();
            worker.send_local_task(async move {
                if let Some(_self) = _self.upgrade() {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 100, 100).await;
                    _self.set_image(result.as_ref());
                }
            });
        }
    }

    pub fn bind(&self, model: &SongModel, worker: Worker, show_cover: bool) {
        let widget = imp::SongWidget::from_instance(self);

        model.bind_title(&*widget.song_title, "label");
        model.bind_artist(&*widget.song_artist, "label");
        model.bind_duration(&*widget.song_length, "label");
        if show_cover {
            self.bind_art(model, worker);
        } else {
            model.bind_index(&*widget.song_index, "label");
        }

        self.set_playing(model.get_playing());
        model.connect_playing_local(clone!(@weak self as _self => move |song| {
            _self.set_playing(song.get_playing());
        }));

        self.set_selected(model.get_selected());
        model.connect_selected_local(clone!(@weak self as _self => move |song| {
            _self.set_selected(song.get_selected());
        }));
    }
}
