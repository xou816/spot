use glib::signal;
use gtk::prelude::*;
use gtk::{BinExt, ImageExt, LabelExt, RangeExt, ScaleExt};
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{utils::Clock, EventListener};
use crate::app::AppEvent;
use crate::app::{ActionDispatcher, AppAction, AppModel, AppState};

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
        let state = self.state();
        state.is_playing && state.current_song_id.is_some()
    }

    pub fn current_song_duration(&self) -> Option<f64> {
        self.state().current_song().map(|s| s.duration as f64)
    }

    pub fn play_next_song(&self) {
        self.dispatcher.dispatch(AppAction::Next);
    }

    pub fn play_prev_song(&self) {
        self.dispatcher.dispatch(AppAction::Previous);
    }

    pub fn toggle_playback(&self) {
        self.dispatcher.dispatch(AppAction::TogglePlay);
    }

    pub fn toggle_shuffle(&self) {
        self.dispatcher.dispatch(AppAction::ToggleShuffle);
    }

    pub fn seek_to(&self, position: u32) {
        self.dispatcher.dispatch(AppAction::Seek(position));
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
    clock: Clock,
}

impl PlaybackControl {
    pub fn new(model: PlaybackControlModel, widget: PlaybackControlWidget) -> Self {
        let model = Rc::new(model);

        widget
            .seek_bar
            .connect_format_value(|_, value| Self::format_duration(value));

        widget.seek_bar.connect_change_value(
            clone!(@weak model => @default-return signal::Inhibit(false), move |_, _, requested| {
                model.seek_to(requested as u32);
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
            AppEvent::TrackPaused | AppEvent::TrackResumed => {
                self.toggle_playing();
            }
            AppEvent::TrackChanged(_) => {
                self.update_current_info();
            }
            AppEvent::SeekSynced(pos) => {
                self.sync_seek(*pos);
            }
            _ => {}
        }
    }
}
