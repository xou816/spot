use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::app::state::RepeatMode;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/playback_controls.ui")]
    pub struct PlaybackControlsWidget {
        #[template_child]
        pub play_pause: TemplateChild<gtk::Button>,

        #[template_child]
        pub next: TemplateChild<gtk::Button>,

        #[template_child]
        pub prev: TemplateChild<gtk::Button>,

        #[template_child]
        pub shuffle: TemplateChild<gtk::ToggleButton>,

        #[template_child]
        pub repeat: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackControlsWidget {
        const NAME: &'static str = "PlaybackControlsWidget";
        type Type = super::PlaybackControlsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackControlsWidget {}
    impl WidgetImpl for PlaybackControlsWidget {}
    impl BoxImpl for PlaybackControlsWidget {}
}

glib::wrapper! {
    pub struct PlaybackControlsWidget(ObjectSubclass<imp::PlaybackControlsWidget>) @extends gtk::Widget, gtk::Box;
}

impl PlaybackControlsWidget {
    pub fn set_playing(&self, is_playing: bool) {
        let playback_icon = if is_playing {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        };

        let tooltip_text = if is_playing {
            Some("Pause")
        } else {
            Some("Play")
        };

        let playback_control = imp::PlaybackControlsWidget::from_instance(self);

        playback_control.play_pause.set_icon_name(playback_icon);
        playback_control.play_pause.set_tooltip_text(tooltip_text);
    }

    pub fn set_shuffled(&self, shuffled: bool) {
        imp::PlaybackControlsWidget::from_instance(self)
            .shuffle
            .set_active(shuffled);
    }

    pub fn set_repeat_mode(&self, mode: RepeatMode) {
        let repeat_mode_icon = match mode {
            RepeatMode::Song => "media-playlist-repeat-song-symbolic",
            RepeatMode::Playlist => "media-playlist-repeat-symbolic",
            RepeatMode::None => "media-playlist-consecutive-symbolic",
        };

        imp::PlaybackControlsWidget::from_instance(self)
            .repeat
            .set_icon_name(repeat_mode_icon);
    }

    pub fn connect_play_pause<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::PlaybackControlsWidget::from_instance(self)
            .play_pause
            .connect_clicked(move |_| f());
    }

    pub fn connect_prev<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::PlaybackControlsWidget::from_instance(self)
            .prev
            .connect_clicked(move |_| f());
    }

    pub fn connect_next<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::PlaybackControlsWidget::from_instance(self)
            .next
            .connect_clicked(move |_| f());
    }

    pub fn connect_shuffle<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::PlaybackControlsWidget::from_instance(self)
            .shuffle
            .connect_clicked(move |_| f());
    }

    pub fn connect_repeat<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::PlaybackControlsWidget::from_instance(self)
            .repeat
            .connect_clicked(move |_| f());
    }
}
