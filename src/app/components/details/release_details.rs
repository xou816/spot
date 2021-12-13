use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use libadwaita::subclass::prelude::*;

use crate::app::components::labels;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/release_details.ui")]
    pub struct ReleaseDetailsWindow {
        #[template_child]
        pub album_artist: TemplateChild<libadwaita::WindowTitle>,

        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        #[template_child]
        pub release: TemplateChild<gtk::Label>,

        #[template_child]
        pub tracks: TemplateChild<gtk::Label>,

        #[template_child]
        pub duration: TemplateChild<gtk::Label>,

        #[template_child]
        pub copyright: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ReleaseDetailsWindow {
        const NAME: &'static str = "ReleaseDetailsWindow";
        type Type = super::ReleaseDetailsWindow;
        type ParentType = libadwaita::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ReleaseDetailsWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for ReleaseDetailsWindow {}
    impl AdwWindowImpl for ReleaseDetailsWindow {}
    impl WindowImpl for ReleaseDetailsWindow {}
}

glib::wrapper! {
    pub struct ReleaseDetailsWindow(ObjectSubclass<imp::ReleaseDetailsWindow>) @extends gtk::Widget, libadwaita::Window, libadwaita::PreferencesWindow;
}

impl ReleaseDetailsWindow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of ReleaseDetailsWindow")
    }

    fn widget(&self) -> &imp::ReleaseDetailsWindow {
        imp::ReleaseDetailsWindow::from_instance(self)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_details(
        &self,
        album: &str,
        artist: &str,
        label: &str,
        release_date: &str,
        track_count: usize,
        duration: &str,
        copyright: &str,
    ) {
        let widget = self.widget();

        widget
            .album_artist
            .set_title(&labels::album_by_artist_label(album, artist));

        widget.label.set_text(label);
        widget.release.set_text(release_date);
        widget.tracks.set_text(&track_count.to_string());
        widget.duration.set_text(duration);
        widget.copyright.set_text(copyright);
    }
}
