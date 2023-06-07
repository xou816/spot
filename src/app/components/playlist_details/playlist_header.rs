use crate::app::components::display_add_css_provider;
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

const CSS_RO_ENTRY: &str = "playlist__title-entry--ro";

mod imp {

    use glib::{ParamSpec, Properties};
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/dev/alextren/Spot/components/playlist_header.ui")]
    #[properties(wrapper_type = super::PlaylistHeaderWidget)]
    pub struct PlaylistHeaderWidget {
        #[template_child]
        pub playlist_label_entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub playlist_image_box: TemplateChild<gtk::Box>,

        #[template_child]
        pub playlist_art: TemplateChild<gtk::Image>,

        #[template_child]
        pub playlist_info: TemplateChild<gtk::Box>,

        #[template_child]
        pub author_button: TemplateChild<gtk::LinkButton>,

        #[template_child]
        pub author_button_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub play_button: TemplateChild<gtk::Button>,

        #[property(get, set, name = "original-entry-text")]
        pub original_entry_text: RefCell<String>,
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

    impl ObjectImpl for PlaylistHeaderWidget {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for PlaylistHeaderWidget {}
    impl BoxImpl for PlaylistHeaderWidget {}
}

glib::wrapper! {
    pub struct PlaylistHeaderWidget(ObjectSubclass<imp::PlaylistHeaderWidget>) @extends gtk::Widget, gtk::Box;
}

impl PlaylistHeaderWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn connect_owner_clicked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().author_button.connect_activate_link(move |_| {
            f();
            glib::signal::Inhibit(true)
        });
    }

    pub fn connect_play<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().play_button.connect_clicked(move |_| f());
    }

    pub fn reset_playlist_name(&self) {
        self.imp()
            .playlist_label_entry
            .set_text(&self.original_entry_text());
    }

    pub fn get_edited_playlist_name(&self) -> String {
        self.imp().playlist_label_entry.text().to_string()
    }

    pub fn set_artwork(&self, art: &gdk_pixbuf::Pixbuf) {
        self.imp().playlist_art.set_from_pixbuf(Some(art));
    }

    pub fn set_info(&self, playlist: &str, owner: &str) {
        let widget = self.imp();
        self.set_original_entry_text(playlist);
        widget.playlist_label_entry.set_text(playlist);
        widget
            .playlist_label_entry
            .set_placeholder_text(Some(playlist));
        widget.author_button_label.set_label(owner);
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

    pub fn set_centered(&self) {
        let widget = self.imp();
        widget.playlist_info.set_halign(gtk::Align::Center);
        widget.play_button.set_margin_end(0);
        widget.playlist_info.set_margin_start(0);
        widget.playlist_image_box.set_margin_start(0);
        widget.playlist_label_entry.set_xalign(0.5);
        widget.author_button.set_halign(gtk::Align::Center);
    }

    pub fn set_editing(&self, editing: bool) {
        let widget = self.imp();
        widget.playlist_label_entry.set_can_focus(editing);
        widget.playlist_label_entry.set_editable(editing);
        if editing {
            widget.playlist_label_entry.remove_css_class(CSS_RO_ENTRY);
        } else {
            widget.playlist_label_entry.add_css_class(CSS_RO_ENTRY);
        }
    }

    pub fn entry(&self) -> &gtk::Entry {
        self.imp().playlist_label_entry.as_ref()
    }

    pub fn set_grows_automatically(&self) {
        let entry: &gtk::Entry = &self.imp().playlist_label_entry;
        entry
            .bind_property("text", entry, "width-chars")
            .transform_to(|_, text: &str| Some(text.len() as i32))
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
    }
}
