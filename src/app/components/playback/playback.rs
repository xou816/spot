use gtk::prelude::*;
use gtk::{BinExt, ImageExt, LabelExt, RangeExt, ScaleExt};
use std::cell::Cell;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::AppEvent;

struct Clock {
    interval_ms: u32,
    source: Cell<Option<glib::source::SourceId>>,
}

impl Clock {
    fn new() -> Self {
        Self {
            interval_ms: 1000,
            source: Cell::new(None),
        }
    }

    fn start<F: Fn() + 'static>(&self, tick: F) {
        let new_source = Some(glib::timeout_add_local(self.interval_ms, move || {
            tick();
            glib::Continue(true)
        }));
        if let Some(previous_source) = self.source.replace(new_source) {
            glib::source_remove(previous_source);
        }
    }

    fn stop(&self) {
        let new_source = None;
        if let Some(previous_source) = self.source.replace(new_source) {
            glib::source_remove(previous_source);
        }
    }
}

pub struct PlaybackWidget {
    play_button: gtk::Button,
    shuffle_button: gtk::ToggleButton,
    current_song_image: gtk::Image,
    current_song_info: gtk::Label,
    seek_bar: gtk::Scale,
    track_duration: gtk::Label,
    next: gtk::Button,
    prev: gtk::Button,
}

impl PlaybackWidget {
    pub fn new(
        play_button: gtk::Button,
        shuffle_button: gtk::ToggleButton,
        current_song_image: gtk::Image,
        current_song_info: gtk::Label,
        seek_bar: gtk::Scale,
        track_duration: gtk::Label,
        next: gtk::Button,
        prev: gtk::Button,
    ) -> Self {
        seek_bar.connect_format_value(|_, value| Self::format_duration(value));

        Self {
            play_button,
            shuffle_button,
            current_song_image,
            current_song_info,
            seek_bar,
            track_duration,
            next,
            prev,
        }
    }

    fn set_playing(&self, is_playing: bool) {
        let playback_image = if is_playing {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        };

        self.play_button
            .get_child()
            .and_then(|child| child.downcast::<gtk::Image>().ok())
            .map(|image| {
                image.set_from_icon_name(Some(playback_image), gtk::IconSize::Button);
            })
            .expect("error updating icon");
    }

    fn format_duration(duration: f64) -> String {
        let seconds = (duration / 1000.0) as i32;
        let minutes = seconds.div_euclid(60);
        let seconds = seconds.rem_euclid(60);
        format!("{}:{:02}", minutes, seconds)
    }

    fn set_duration(&self, duration: f64) {
        self.seek_bar.set_draw_value(true);
        self.seek_bar.set_range(0.0, duration);
        self.seek_bar.set_value(0.0);
        self.track_duration.show();
        self.track_duration
            .set_text(&Self::format_duration(duration));
    }
}

use super::PlaybackModel;

pub struct Playback {
    model: Rc<PlaybackModel>,
    worker: Worker,
    widget: PlaybackWidget,
    clock: Clock,
}

impl Playback {
    pub fn new(model: PlaybackModel, worker: Worker, widget: PlaybackWidget) -> Self {
        let model = Rc::new(model);

        let weak_model = Rc::downgrade(&model);
        widget
            .seek_bar
            .connect_change_value(move |_, _, requested| {
                if let Some(model) = weak_model.upgrade() {
                    model.seek_to(requested as u32)
                }
                glib::signal::Inhibit(false)
            });

        let weak_model = Rc::downgrade(&model);
        widget.play_button.connect_clicked(move |_| {
            if let Some(model) = weak_model.upgrade() {
                model.toggle_playback()
            }
        });

        let weak_model = Rc::downgrade(&model);
        widget.next.connect_clicked(move |_| {
            if let Some(model) = weak_model.upgrade() {
                model.play_next_song()
            }
        });

        let weak_model = Rc::downgrade(&model);
        widget.prev.connect_clicked(move |_| {
            if let Some(model) = weak_model.upgrade() {
                model.play_prev_song()
            }
        });

        let weak_model = Rc::downgrade(&model);
        widget.shuffle_button.connect_clicked(move |_| {
            if let Some(model) = weak_model.upgrade() {
                model.toggle_shuffle();
            }
        });

        Self {
            model,
            worker,
            widget,
            clock: Clock::new(),
        }
    }

    fn toggle_playing(&self) {
        let is_playing = self.model.is_playing();
        self.widget.set_playing(is_playing);

        if is_playing {
            let seek_bar = self.widget.seek_bar.clone();
            self.clock.start(move || {
                let value = seek_bar.get_value();
                seek_bar.set_value(value + 1000.0);
            });
        } else {
            self.clock.stop();
        }
    }

    fn update_current_info(&self) {
        if let Some(song) = self.model.current_song() {
            let title = glib::markup_escape_text(&song.title);
            let artist = glib::markup_escape_text(&song.artist);
            let label = format!("<b>{}</b>\n{}", title.as_str(), artist.as_str());
            self.widget.current_song_info.set_label(&label[..]);

            let image = self.widget.current_song_image.clone();
            if let Some(url) = song.art {
                self.worker.send_local_task(async move {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 48, 48).await;
                    image.set_from_pixbuf(result.as_ref());
                });
            }

            self.widget.set_duration(song.duration as f64);
        }
    }

    fn sync_seek(&self, pos: u32) {
        self.widget.seek_bar.set_value(pos as f64);
    }
}

impl EventListener for Playback {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::TrackPaused | AppEvent::TrackResumed => {
                self.toggle_playing();
            }
            AppEvent::TrackChanged(_) => {
                self.update_current_info();
                self.toggle_playing();
            }
            AppEvent::SeekSynced(pos) => {
                self.sync_seek(*pos);
            }
            _ => {}
        }
    }
}
