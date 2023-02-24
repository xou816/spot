use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/playback_info.ui")]
    pub struct PlaybackInfoWidget {
        #[template_child]
        pub playing_image: TemplateChild<gtk::Image>,

        #[template_child]
        pub current_song_info: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackInfoWidget {
        const NAME: &'static str = "PlaybackInfoWidget";
        type Type = super::PlaybackInfoWidget;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackInfoWidget {}
    impl WidgetImpl for PlaybackInfoWidget {}
    impl ButtonImpl for PlaybackInfoWidget {}
}

glib::wrapper! {
    pub struct PlaybackInfoWidget(ObjectSubclass<imp::PlaybackInfoWidget>) @extends gtk::Widget, gtk::Button;
}

impl PlaybackInfoWidget {
    pub fn set_title_and_artist(&self, title: &str, artist: &str) {
        let widget = self.imp();
        let title = glib::markup_escape_text(title);
        let artist = glib::markup_escape_text(artist);
        let label = format!("<b>{}</b>\n{}", title.as_str(), artist.as_str());
        widget.current_song_info.set_label(&label[..]);
    }

    pub fn reset_info(&self) {
        let widget = self.imp();
        widget
            .current_song_info
            // translators: Short text displayed instead of a song title when nothing plays
            .set_label(&gettext("No song playing"));
        widget
            .playing_image
            .set_from_icon_name(Some("emblem-music-symbolic"));
        widget
            .playing_image
            .set_from_icon_name(Some("emblem-music-symbolic"));
    }

    pub fn set_info_visible(&self, visible: bool) {
        self.imp().current_song_info.set_visible(visible);
    }

    pub fn set_artwork(&self, art: &gdk_pixbuf::Pixbuf) {
        self.imp().playing_image.set_from_pixbuf(Some(art));
    }
}
