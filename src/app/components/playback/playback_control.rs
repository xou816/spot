use glib::signal;
use gtk::prelude::*;
use gtk::{BinExt, ImageExt, LabelExt, RangeExt, ScaleExt};
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{
    utils::{Clock, Debouncer},
    EventListener,
};
use crate::app::state::{PlaybackAction, PlaybackEvent};
use crate::app::{ActionDispatcher, AppEvent, AppModel, AppState};

pub struct PlaybackControlModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl PlaybackControlModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> impl Deref<Target = AppState> + '_ {
        self.app_model.get_state()
    }

    pub fn is_playing(&self) -> bool {
        self.state().playback.is_playing()
    }

    pub fn current_song_duration(&self) -> Option<f64> {
        self.state()
            .playback
            .current_song()
            .map(|s| s.duration as f64)
    }

    pub fn play_next_song(&self) {
        self.dispatcher.dispatch(PlaybackAction::Next.into());
    }

    pub fn play_prev_song(&self) {
        self.dispatcher.dispatch(PlaybackAction::Previous.into());
    }

    pub fn toggle_playback(&self) {
        self.dispatcher.dispatch(PlaybackAction::TogglePlay.into());
    }

    pub fn toggle_shuffle(&self) {
        self.dispatcher
            .dispatch(PlaybackAction::ToggleShuffle.into());
    }

    pub fn seek_to(&self, position: u32) {
        self.dispatcher
            .dispatch(PlaybackAction::Seek(position).into());
    }
}

pub struct PlaybackControlWidget {
    play_button: gtk::Button,
    shuffle_button: gtk::ToggleButton,
    seek_bar: gtk::Scale,
    track_duration: gtk::Label,
    next: gtk::Button,
    prev: gtk::Button,
}

impl PlaybackControlWidget {
    pub fn new(
        play_button: gtk::Button,
        shuffle_button: gtk::ToggleButton,
        seek_bar: gtk::Scale,
        track_duration: gtk::Label,
        next: gtk::Button,
        prev: gtk::Button,
    ) -> Self {
        Self {
            play_button,
            shuffle_button,
            seek_bar,
            track_duration,
            next,
            prev,
        }
    }
}

pub struct PlaybackControl {
    model: Rc<PlaybackControlModel>,
    widget: PlaybackControlWidget,
    _debouncer: Debouncer,
    clock: Clock,
}

impl PlaybackControl {
    pub fn new(model: PlaybackControlModel, widget: PlaybackControlWidget) -> Self {
        let model = Rc::new(model);

        widget
            .seek_bar
            .connect_format_value(|_, value| Self::format_duration(value));

        let debouncer = Debouncer::new();
        let debouncer_clone = debouncer.clone();
        widget.seek_bar.connect_change_value(
            clone!(@weak model => @default-return signal::Inhibit(false), move |_, _, requested| {
                debouncer_clone.debounce(200, move || {
                    model.seek_to(requested as u32);
                });
                signal::Inhibit(false)
            }),
        );

        widget
            .play_button
            .connect_clicked(clone!(@weak model => move |_| {
                model.toggle_playback();
            }));

        widget.next.connect_clicked(clone!(@weak model => move |_| {
            model.play_next_song();
        }));

        widget.prev.connect_clicked(clone!(@weak model => move |_| {
            model.play_prev_song()
        }));

        widget
            .shuffle_button
            .connect_clicked(clone!(@weak model => move |_| {
                model.toggle_shuffle();
            }));

        Self {
            model,
            widget,
            _debouncer: debouncer,
            clock: Clock::new(),
        }
    }

    fn set_playing(&self, is_playing: bool) {
        let playback_image = if is_playing {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        };

        self.widget
            .play_button
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

    fn toggle_playing(&self) {
        let is_playing = self.model.is_playing();
        self.set_playing(is_playing);

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
        if let Some(duration) = self.model.current_song_duration() {
            self.widget.seek_bar.set_draw_value(true);
            self.widget.seek_bar.set_range(0.0, duration);
            self.widget.seek_bar.set_value(0.0);
            self.widget.track_duration.show();
            self.widget
                .track_duration
                .set_text(&Self::format_duration(duration));
        }
    }

    fn sync_seek(&self, pos: u32) {
        self.widget.seek_bar.set_value(pos as f64);
    }
}

impl EventListener for PlaybackControl {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::PlaybackEvent(PlaybackEvent::TrackPaused)
            | AppEvent::PlaybackEvent(PlaybackEvent::TrackResumed) => {
                self.toggle_playing();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) => {
                self.update_current_info();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::SeekSynced(pos)) => {
                self.sync_seek(*pos);
            }
            _ => {}
        }
    }
}
