use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::app::components::display_add_css_provider;
use crate::app::components::utils::{format_duration, Clock, Debouncer};
use crate::app::loader::ImageLoader;
use crate::app::models::RepeatMode;
use crate::app::Worker;

use super::playback_controls::PlaybackControlsWidget;
use super::playback_info::PlaybackInfoWidget;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/playback_widget.ui")]
    pub struct PlaybackWidget {
        #[template_child]
        pub controls: TemplateChild<PlaybackControlsWidget>,

        #[template_child]
        pub controls_mobile: TemplateChild<PlaybackControlsWidget>,

        #[template_child]
        pub now_playing: TemplateChild<PlaybackInfoWidget>,

        #[template_child]
        pub now_playing_mobile: TemplateChild<PlaybackInfoWidget>,

        #[template_child]
        pub seek_bar: TemplateChild<gtk::Scale>,

        #[template_child]
        pub track_position: TemplateChild<gtk::Label>,

        #[template_child]
        pub track_duration: TemplateChild<gtk::Label>,

        pub clock: Clock,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackWidget {
        const NAME: &'static str = "PlaybackWidget";
        type Type = super::PlaybackWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.now_playing_mobile.set_info_visible(false);
            self.now_playing.set_info_visible(true);
            display_add_css_provider(resource!("/components/playback.css"));
        }
    }

    impl WidgetImpl for PlaybackWidget {}
    impl BoxImpl for PlaybackWidget {}
}

glib::wrapper! {
    pub struct PlaybackWidget(ObjectSubclass<imp::PlaybackWidget>) @extends gtk::Widget, gtk::Box;
}

impl PlaybackWidget {
    pub fn set_title_and_artist(&self, title: &str, artist: &str) {
        let widget = self.imp();
        widget.now_playing.set_title_and_artist(title, artist);
    }

    pub fn reset_info(&self) {
        let widget = self.imp();
        widget.now_playing.reset_info();
        widget.now_playing_mobile.reset_info();
        self.set_song_duration(None);
    }

    fn set_artwork(&self, image: &gdk_pixbuf::Pixbuf) {
        let widget = self.imp();
        widget.now_playing.set_artwork(image);
        widget.now_playing_mobile.set_artwork(image);
    }

    pub fn set_artwork_from_url(&self, url: String, worker: &Worker) {
        let weak_self = self.downgrade();
        worker.send_local_task(async move {
            let loader = ImageLoader::new();
            let result = loader.load_remote(&url, "jpg", 64, 64).await;
            if let (Some(ref _self), Some(ref result)) = (weak_self.upgrade(), result) {
                _self.set_artwork(result);
            }
        });
    }

    pub fn set_song_duration(&self, duration: Option<f64>) {
        let widget = self.imp();
        let class = "seek-bar--active";
        if let Some(duration) = duration {
            self.add_css_class(class);
            widget.seek_bar.set_range(0.0, duration);
            widget.seek_bar.set_value(0.0);
            widget.track_position.set_text("0∶00");
            widget
                .track_duration
                .set_text(&format!(" / {}", format_duration(duration)));
            widget.track_position.set_visible(true);
            widget.track_duration.set_visible(true);
        } else {
            self.remove_css_class(class);
            widget.seek_bar.set_range(0.0, 0.0);
            widget.track_position.set_visible(false);
            widget.track_duration.set_visible(false);
        }
    }

    pub fn set_seek_position(&self, pos: f64) {
        let widget = self.imp();
        widget.seek_bar.set_value(pos);
        widget.track_position.set_text(&format_duration(pos));
    }

    pub fn increment_seek_position(&self) {
        let value = self.imp().seek_bar.value() + 1_000.0;
        self.set_seek_position(value);
    }

    pub fn set_volume(&self, volume: f64) {
        let widget = imp::PlaybackWidget::from_instance(self);
        widget.controls.set_volume(volume);
        widget.controls_mobile.set_volume(volume);
    }

    pub fn connect_now_playing_clicked<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        let widget = self.imp();
        let f_clone = f.clone();
        widget.now_playing.connect_clicked(move |_| f_clone());
        widget.now_playing_mobile.connect_clicked(move |_| f());
    }

    pub fn connect_seek<Seek>(&self, seek: Seek)
    where
        Seek: Fn(u32) + Clone + 'static,
    {
        let debouncer = Debouncer::new();
        let widget = self.imp();
        widget.seek_bar.set_increments(5_000.0, 10_000.0);
        widget.seek_bar.connect_change_value(
            clone!(@weak self as _self => @default-return glib::signal::Inhibit(false), move |_, _, requested| {
                _self.imp()
                    .track_position
                    .set_text(&format_duration(requested));
                let seek = seek.clone();
                debouncer.debounce(200, move || seek(requested as u32));
                glib::signal::Inhibit(false)
            }),
        );
    }

    pub fn set_playing(&self, is_playing: bool) {
        let widget = self.imp();
        widget.controls.set_playing(is_playing);
        widget.controls_mobile.set_playing(is_playing);
        if is_playing {
            widget
                .clock
                .start(clone!(@weak self as _self => move || _self.increment_seek_position()));
        } else {
            widget.clock.stop();
        }
    }

    pub fn set_repeat_mode(&self, mode: RepeatMode) {
        let widget = self.imp();
        widget.controls.set_repeat_mode(mode);
        widget.controls_mobile.set_repeat_mode(mode);
    }

    pub fn set_shuffled(&self, shuffled: bool) {
        let widget = self.imp();
        widget.controls.set_shuffled(shuffled);
        widget.controls_mobile.set_shuffled(shuffled);
    }

    pub fn set_seekbar_visible(&self, visible: bool) {
        let widget = self.imp();
        widget.seek_bar.set_visible(visible);
    }

    pub fn connect_play_pause<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        let widget = self.imp();
        widget.controls.connect_play_pause(f.clone());
        widget.controls_mobile.connect_play_pause(f);
    }

    pub fn connect_prev<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        let widget = self.imp();
        widget.controls.connect_prev(f.clone());
        widget.controls_mobile.connect_prev(f);
    }

    pub fn connect_next<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        let widget = self.imp();
        widget.controls.connect_next(f.clone());
        widget.controls_mobile.connect_next(f);
    }

    pub fn connect_shuffle<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        let widget = self.imp();
        widget.controls.connect_shuffle(f.clone());
        widget.controls_mobile.connect_shuffle(f);
    }

    pub fn connect_repeat<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        let widget = self.imp();
        widget.controls.connect_repeat(f.clone());
        widget.controls_mobile.connect_repeat(f);
    }

    pub fn connect_volume_changed<F>(&self, f: F)
    where
        F: Fn(f64) + Clone + 'static,
    {
        let widget = imp::PlaybackWidget::from_instance(self);
        widget.controls.connect_volume_changed(f.clone());
        widget.controls_mobile.connect_volume_changed(f);
    }
}
