use crate::app::components::display_add_css_provider;
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {

    use std::cell::Cell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::AlbumHeaderWidget)]
    #[template(resource = "/dev/alextren/Spot/components/album_header.ui")]
    pub struct AlbumHeaderWidget {
        #[template_child]
        pub album_overlay: TemplateChild<gtk::Overlay>,

        #[template_child]
        pub album_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub album_art: TemplateChild<gtk::Image>,

        #[template_child]
        pub button_box: TemplateChild<gtk::Box>,

        #[template_child]
        pub like_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub play_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub info_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub album_info: TemplateChild<gtk::Box>,

        #[template_child]
        pub artist_button: TemplateChild<gtk::LinkButton>,

        #[template_child]
        pub artist_button_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub year_label: TemplateChild<gtk::Label>,

        #[property(get, set = Self::set_vertical, name = "vertical-layout")]
        pub vertical_layout: Cell<bool>,
    }

    impl AlbumHeaderWidget {
        pub fn set_vertical(&self, vertical: bool) {
            let self_ = self.obj();
            let box_ = self_.upcast_ref::<gtk::Box>();
            if vertical {
                box_.set_orientation(gtk::Orientation::Vertical);
                box_.set_spacing(12);
                self.album_label.set_halign(gtk::Align::Center);
                self.album_label.set_justify(gtk::Justification::Center);
                self.artist_button.set_halign(gtk::Align::Center);
                self.year_label.set_halign(gtk::Align::Center);
                self.button_box.set_halign(gtk::Align::Center);
                self.album_overlay.set_margin_start(0);
                self.button_box.set_margin_end(0);
                self.album_info.set_margin_start(0);
            } else {
                box_.set_orientation(gtk::Orientation::Horizontal);
                box_.set_spacing(0);
                self.album_label.set_halign(gtk::Align::Start);
                self.album_label.set_justify(gtk::Justification::Left);
                self.artist_button.set_halign(gtk::Align::Start);
                self.year_label.set_halign(gtk::Align::Start);
                self.button_box.set_halign(gtk::Align::Start);
                self.album_overlay.set_margin_start(6);
                self.button_box.set_margin_end(6);
                self.album_info.set_margin_start(18);
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumHeaderWidget {
        const NAME: &'static str = "AlbumHeaderWidget";
        type Type = super::AlbumHeaderWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            display_add_css_provider(resource!("/components/album_header.css"));
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AlbumHeaderWidget {}
    impl WidgetImpl for AlbumHeaderWidget {}
    impl BoxImpl for AlbumHeaderWidget {}
}

glib::wrapper! {
    pub struct AlbumHeaderWidget(ObjectSubclass<imp::AlbumHeaderWidget>) @extends gtk::Widget, gtk::Box;
}

impl Default for AlbumHeaderWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl AlbumHeaderWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn connect_play<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().play_button.connect_clicked(move |_| f());
    }

    pub fn connect_liked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().like_button.connect_clicked(move |_| f());
    }

    pub fn connect_info<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().info_button.connect_clicked(move |_| f());
    }

    pub fn connect_artist_clicked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().artist_button.connect_activate_link(move |_| {
            f();
            glib::Propagation::Stop
        });
    }

    pub fn set_liked(&self, is_liked: bool) {
        self.imp().like_button.set_icon_name(if is_liked {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        });
    }

    pub fn set_playing(&self, is_playing: bool) {
        let playback_icon = if is_playing {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        };

        let translated_tooltip = if is_playing {
            gettext("Pause")
        } else {
            gettext("Play")
        };
        let tooltip_text = Some(translated_tooltip.as_str());

        self.imp().play_button.set_icon_name(playback_icon);
        self.imp().play_button.set_tooltip_text(tooltip_text);
    }

    pub fn set_artwork(&self, pixbuf: &gdk_pixbuf::Pixbuf) {
        let texture = gdk::Texture::for_pixbuf(pixbuf);
        self.imp().album_art.set_paintable(Some(&texture));
    }

    pub fn set_album_and_artist_and_year(&self, album: &str, artist: &str, year: Option<u32>) {
        let widget = self.imp();
        widget.album_label.set_label(album);
        widget.artist_button_label.set_label(artist);
        match year {
            Some(year) => widget.year_label.set_label(&year.to_string()),
            None => widget.year_label.set_visible(false),
        }
    }
}
