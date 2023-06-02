use crate::app::components::display_add_css_provider;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gettextrs::gettext;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/album_header.ui")]
    pub struct AlbumHeaderWidget {
        #[template_child]
        pub album_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub album_art: TemplateChild<gtk::Image>,

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

    impl ObjectImpl for AlbumHeaderWidget {}
    impl WidgetImpl for AlbumHeaderWidget {}
    impl BoxImpl for AlbumHeaderWidget {}
}

glib::wrapper! {
    pub struct AlbumHeaderWidget(ObjectSubclass<imp::AlbumHeaderWidget>) @extends gtk::Widget, gtk::Box;
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
            glib::signal::Inhibit(true)
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
        let playback_control = imp::AlbumHeaderWidget::from_obj(self);

        playback_control.play_button.set_icon_name(playback_icon);
        playback_control.play_button.set_tooltip_text(tooltip_text);
    }

    pub fn set_artwork(&self, art: &gdk_pixbuf::Pixbuf) {
        self.imp().album_art.set_from_pixbuf(Some(art));
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

    pub fn set_centered(&self) {
        let widget = self.imp();
        widget.album_label.set_halign(gtk::Align::Center);
        widget.album_label.set_justify(gtk::Justification::Center);
        widget.artist_button.set_halign(gtk::Align::Center);
        widget.year_label.set_halign(gtk::Align::Center);
    }
}
