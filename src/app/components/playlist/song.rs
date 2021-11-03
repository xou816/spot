use crate::app::components::display_add_css_provider;
use crate::app::loader::ImageLoader;
use crate::app::models::SongModel;

use crate::app::Worker;
use gio::MenuModel;
use glib::subclass::InitializingObject;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    const SONG_CLASS: &str = "song--playing";

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
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 2] = [
            glib::ParamSpecBoolean::builder("playing").build(),
            glib::ParamSpecBoolean::builder("selected").build()
        ];
    }

    impl ObjectImpl for SongWidget {
        fn properties() -> &'static [glib::ParamSpec] {
            &*PROPERTIES
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "playing" => {
                    let is_playing = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    let context = self.obj().style_context();
                    if is_playing {
                        context.add_class(SONG_CLASS);
                    } else {
                        context.remove_class(SONG_CLASS);
                    }
                }
                "selected" => {
                    let is_selected = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.song_checkbox.set_active(is_selected);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "playing" => self.obj().style_context().has_class(SONG_CLASS).to_value(),
                "selected" => self.song_checkbox.is_active().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.song_checkbox.set_sensitive(false);
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
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
        glib::Object::new()
    }

    pub fn set_actions(&self, actions: Option<&gio::ActionGroup>) {
        self.insert_action_group("song", actions);
    }

    pub fn set_menu(&self, menu: Option<&MenuModel>) {
        if menu.is_some() {
            let widget = self.imp();
            widget.menu_btn.set_menu_model(menu);
            widget
                .menu_btn
                .style_context()
                .add_class("song__menu--enabled");
        }
    }

    fn set_show_cover(&self, show_cover: bool) {
        let song_class = "song--cover";
        let context = self.style_context();
        if show_cover {
            context.add_class(song_class);
        } else {
            context.remove_class(song_class);
        }
    }

    fn set_image(&self, pixbuf: Option<&gdk_pixbuf::Pixbuf>) {
        self.imp().song_cover.set_from_pixbuf(pixbuf);
    }

    pub fn set_art(&self, model: &SongModel, worker: Worker) {
        if let Some(url) = model.description().art.clone() {
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
        let widget = self.imp();

        model.bind_title(&*widget.song_title, "label");
        model.bind_artist(&*widget.song_artist, "label");
        model.bind_duration(&*widget.song_length, "label");
        model.bind_playing(self, "playing");
        model.bind_selected(self, "selected");

        self.set_show_cover(show_cover);
        if show_cover {
            self.set_art(model, worker);
        } else {
            model.bind_index(&*widget.song_index, "label");
        }
    }
}
