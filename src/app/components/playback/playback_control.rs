use glib::signal;
use gtk::prelude::*;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::utils::format_duration;
use crate::app::components::{
    utils::{Clock, Debouncer},
    EventListener,
};
use crate::app::state::{PlaybackAction, PlaybackEvent, RepeatMode};
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

    pub fn toggle_repeat(&self) {
        self.dispatcher
            .dispatch(PlaybackAction::ToggleRepeat.into());
    }

    pub fn seek_to(&self, position: u32) {
        self.dispatcher
            .dispatch(PlaybackAction::Seek(position).into());
    }
}

pub struct PlaybackControlWidget {
    play_button: gtk::Button,
    seek_bar: gtk::Scale,
    track_position: gtk::Label,
    track_duration: gtk::Label,
    next: gtk::Button,
    prev: gtk::Button,
    shuffle_button: gtk::Button,
    repeat_button: gtk::Button,
}

impl PlaybackControlWidget {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        play_button: gtk::Button,
        seek_bar: gtk::Scale,
        track_position: gtk::Label,
        track_duration: gtk::Label,
        next: gtk::Button,
        prev: gtk::Button,
        shuffle_button: gtk::Button,
        repeat_button: gtk::Button,
    ) -> Self {
        Self {
            play_button,
            seek_bar,
            track_position,
            track_duration,
            next,
            prev,
            shuffle_button,
            repeat_button,
        }
    }
}

pub struct PlaybackControl {
    model: Rc<PlaybackControlModel>,
    widget: PlaybackControlWidget,
    _debouncer: Debouncer,
    clock: Clock,
}

const STEP: f64 = 5000.0;
impl PlaybackControl {
    pub fn new(model: PlaybackControlModel, widget: PlaybackControlWidget) -> Self {
        let model = Rc::new(model);
        let debouncer = Debouncer::new();
        let debouncer_clone = debouncer.clone();
        let track_position = &widget.track_position;
        widget.seek_bar.set_increments(STEP, STEP);
        widget.seek_bar.connect_change_value(
            clone!(@weak model, @weak track_position => @default-return signal::Inhibit(false), move |_, _, requested| {
                track_position.set_text(&format_duration(requested));
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
            model.play_prev_song();
        }));

        widget
            .shuffle_button
            .connect_clicked(clone!(@weak model => move |_| {
                model.toggle_shuffle();
            }));

        widget
            .repeat_button
            .connect_clicked(clone!(@weak model => move |_| {
                model.toggle_repeat();
            }));

        Self {
            model,
            widget,
            _debouncer: debouncer,
            clock: Clock::new(),
        }
    }

    fn set_playing(&self, is_playing: bool) {
        let playback_icon = if is_playing {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        };

        self.widget.play_button.set_icon_name(playback_icon);
    }

    fn update_repeat(&self, mode: &RepeatMode) {
        let repeat_mode_icon = match mode {
            RepeatMode::Song => "media-playlist-repeat-song-symbolic",
            RepeatMode::Playlist => "media-playlist-repeat-symbolic",
            RepeatMode::None => "media-playlist-consecutive-symbolic",
        };

        self.widget.repeat_button.set_icon_name(repeat_mode_icon);
    }

    fn update_playing(&self) {
        let is_playing = self.model.is_playing();
        self.set_playing(is_playing);

        if is_playing {
            let seek_bar = &self.widget.seek_bar;
            let track_position = &self.widget.track_position;
            self.clock
                .start(clone!(@weak seek_bar, @weak track_position => move || {
                    let value = seek_bar.value() + 1000.0;
                    seek_bar.set_value(value);
                    track_position.set_text(&format_duration(value));
                }));
        } else {
            self.clock.stop();
        }
    }

    fn update_current_info(&self) {
        let class = "seek-bar--active";
        let style_context = self.widget.seek_bar.style_context();
        if let Some(duration) = self.model.current_song_duration() {
            style_context.add_class(class);
            self.widget.seek_bar.set_range(0.0, duration);
            self.widget.seek_bar.set_value(0.0);
            self.widget.track_position.set_text("0:00");
            self.widget
                .track_duration
                .set_text(&format!(" / {}", format_duration(duration)));
            self.widget.track_position.show();
            self.widget.track_duration.show();
        } else {
            style_context.remove_class(class);
            self.widget.seek_bar.set_range(0.0, 0.0);
            self.widget.track_position.hide();
            self.widget.track_duration.hide();
        }
    }

    fn sync_seek(&self, pos: u32) {
        let pos = pos as f64;
        self.widget.seek_bar.set_value(pos);
        self.widget.track_position.set_text(&format_duration(pos));
    }
}

impl EventListener for PlaybackControl {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackPaused)
            | AppEvent::PlaybackEvent(PlaybackEvent::PlaybackResumed) => {
                self.update_playing();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) => {
                self.update_current_info();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::RepeatModeChanged(mode)) => {
                self.update_repeat(mode);
            }
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackStopped) => {
                self.update_playing();
                self.update_current_info();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::SeekSynced(pos))
            | AppEvent::PlaybackEvent(PlaybackEvent::TrackSeeked(pos)) => {
                self.sync_seek(*pos);
            }
            _ => {}
        }
    }
}
