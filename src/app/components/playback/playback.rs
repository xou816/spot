use gtk::prelude::*;
use gtk::{ImageExt, RangeExt, ScaleExt};
use std::rc::Rc;
use std::cell::Cell;

use crate::app::{AppEvent};
use crate::app::components::EventListener;
use crate::app::loader::ImageLoader;
use crate::app::dispatch::Worker;

struct Clock {
    interval_ms: u32,
    source: Cell<Option<glib::source::SourceId>>
}

impl Clock {

    fn new() -> Self {
        Self { interval_ms: 1000, source: Cell::new(None) }
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

use super::PlaybackModel;

pub struct Playback {
    model: Rc<PlaybackModel>,
    worker: Worker,
    play_button: gtk::Button,
    current_song_image: gtk::Image,
    current_song_info: gtk::Label,
    seek_bar: gtk::Scale,
    clock: Clock
}

impl Playback {

    pub fn new(
        model: PlaybackModel,
        worker: Worker,
        play_button: gtk::Button,
        current_song_image: gtk::Image,
        current_song_info: gtk::Label,
        next: gtk::Button,
        prev: gtk::Button,
        seek_bar: gtk::Scale) -> Self {

        let model = Rc::new(model);
        let weak_model = Rc::downgrade(&model);
        seek_bar.connect_change_value(move |_, _, requested| {
            weak_model.upgrade()
                .map(|model| model.seek_to(requested as u32));
            glib::signal::Inhibit(false)
        });

        seek_bar.connect_format_value(|_, value| {
            let seconds = (value/1000.0) as i32;
            let minutes = seconds.div_euclid(60);
            let seconds = seconds.rem_euclid(60);
            format!("{}:{:02}", minutes, seconds)
        });

        let weak_model = Rc::downgrade(&model);
        play_button.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.toggle_playback());
        });

        let weak_model = Rc::downgrade(&model);
        next.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.play_next_song());
        });

        let weak_model = Rc::downgrade(&model);
        prev.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.play_prev_song());
        });

        Self { model, worker, play_button, current_song_image, current_song_info, seek_bar, clock: Clock::new() }
    }

    fn toggle_playing(&self) {

        let is_playing = self.model.is_playing();

        self.play_button.get_children().first()
            .and_then(|child| child.downcast_ref::<gtk::Image>())
            .map(|image| {
                image.set_from_icon_name(
                    Some(playback_image(is_playing)),
                    gtk::IconSize::Button);
            })
            .expect("error updating icon");

        if is_playing {
            let seek_bar = self.seek_bar.clone();
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
            self.current_song_info.set_label(&label[..]);

            let image = self.current_song_image.clone();
            if let Some(url) = song.art {
                let url = url.clone();
                self.worker.send_task(async move {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 48, 48).await;
                    image.set_from_pixbuf(result.as_ref());
                });
            }

            let duration = song.duration as f64;
            self.seek_bar.set_range(0.0, duration);
            self.seek_bar.set_value(0.0);
        }
    }

    fn sync_seek(&self, pos: u32) {
        self.seek_bar.set_value(pos as f64);
    }
}

impl EventListener for Playback {

    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::TrackPaused|AppEvent::TrackResumed => {
                self.toggle_playing();
            },
            AppEvent::TrackChanged(_) => {
                self.update_current_info();
                self.toggle_playing();
            },
            AppEvent::SeekSynced(pos) => {
                self.sync_seek(*pos);
            },
            _ => {}
        }
    }
}

fn playback_image(is_playing: bool) -> &'static str {
    if is_playing {
        "media-playback-pause"
    } else {
        "media-playback-start"
    }
}
