use crate::app::components::display_add_css_provider;
use crate::app::models::SongModel;

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
        pub song_icon: TemplateChild<gtk::Image>,

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

    pub fn for_model(model: SongModel) -> Self {
        let _self = Self::new();
        _self.bind(&model);
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

    pub fn bind(&self, model: &SongModel) {
        let widget = imp::SongWidget::from_instance(self);

        model.bind_index(&*widget.song_index, "label");
        model.bind_title(&*widget.song_title, "label");
        model.bind_artist(&*widget.song_artist, "label");
        model.bind_duration(&*widget.song_length, "label");

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
