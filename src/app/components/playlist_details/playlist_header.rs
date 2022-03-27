use crate::app::components::display_add_css_provider;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/playlist_header.ui")]
    pub struct PlaylistHeaderWidget {
        #[template_child]
        pub playlist_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub playlist_art: TemplateChild<gtk::Image>,

        #[template_child]
        pub playlist_info: TemplateChild<gtk::Box>,

        #[template_child]
        pub author_button: TemplateChild<gtk::LinkButton>,

        #[template_child]
        pub author_button_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub year_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistHeaderWidget {
        const NAME: &'static str = "PlaylistHeaderWidget";
        type Type = super::PlaylistHeaderWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            display_add_css_provider(resource!("/components/playlist_header.css"));
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistHeaderWidget {}
    impl WidgetImpl for PlaylistHeaderWidget {}
    impl BoxImpl for PlaylistHeaderWidget {}
}

glib::wrapper! {
    pub struct PlaylistHeaderWidget(ObjectSubclass<imp::PlaylistHeaderWidget>) @extends gtk::Widget, gtk::Box;
}

impl PlaylistHeaderWidget {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of PlaylistHeaderWidget")
    }

    pub fn widget(&self) -> &imp::PlaylistHeaderWidget {
        imp::PlaylistHeaderWidget::from_instance(self)
    }

    pub fn connect_artist_clicked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().author_button.connect_activate_link(move |_| {
            f();
            glib::signal::Inhibit(true)
        });
    }

    pub fn set_artwork(&self, art: &gdk_pixbuf::Pixbuf) {
        self.widget().playlist_art.set_from_pixbuf(Some(art));
    }

    pub fn set_album_and_artist_and_year(&self, album: &str, artist: &str, year: Option<u32>) {
        let widget = self.widget();
        widget.playlist_label.set_label(album);
        widget.author_button_label.set_label(artist);
        match year {
            Some(year) => widget.year_label.set_label(&year.to_string()),
            None => widget.year_label.hide(),
        }
    }

    pub fn set_centered(&self) {
        let widget = self.widget();
        widget.playlist_label.set_halign(gtk::Align::Center);
        widget
            .playlist_label
            .set_justify(gtk::Justification::Center);
        widget.author_button.set_halign(gtk::Align::Center);
        widget.year_label.set_halign(gtk::Align::Center);
    }
}
