use gtk::prelude::*;
use gtk::ImageExt;
use std::rc::Rc;
use std::cell::RefCell;

use crate::app::{AppAction, SongDescription, Dispatcher};
use crate::app::components::{Component};

pub trait PlaybackState {
    fn is_playing(&self) -> bool;
    fn current_song(&self) -> Option<&SongDescription>;
    fn next_song_action(&self) -> Option<AppAction>;
    fn prev_song_action(&self) -> Option<AppAction>;
}

pub struct Playback<State: 'static> where State: PlaybackState {
    play_button: gtk::Button,
    current_song_info: gtk::Label,
    state: Rc<RefCell<State>>
}

impl<State> Playback<State> where State: PlaybackState {

    pub fn new(
        builder: &gtk::Builder,
        state: Rc<RefCell<State>>,
        dispatcher: Dispatcher) -> Self {

        let play_button: gtk::Button = builder.get_object("play_pause").unwrap();
        let current_song_info: gtk::Label = builder.get_object("current_song_info").unwrap();
        let next: gtk::Button = builder.get_object("next").unwrap();
        let prev: gtk::Button = builder.get_object("prev").unwrap();

        let weak_state = Rc::downgrade(&state);
        let dispatcher_clone = dispatcher.clone();
        play_button.connect_clicked(move |_| {
            weak_state.upgrade()
                .and_then(|state| {
                    let state = state.borrow();
                    dispatcher_clone
                        .dispatch(if state.is_playing() { AppAction::Pause } else { AppAction::Play })
                });
        });

        let weak_state = Rc::downgrade(&state);
        let dispatcher_clone = dispatcher.clone();
        next.connect_clicked(move |_| {
            weak_state.upgrade()
                .and_then(|state| state.borrow().next_song_action())
                .and_then(|action| dispatcher_clone.dispatch(action));
        });

        let weak_state = Rc::downgrade(&state);
        let dispatcher_clone = dispatcher.clone();
        prev.connect_clicked(move |_| {
            weak_state.upgrade()
                .and_then(|state| state.borrow().prev_song_action())
                .and_then(|action| dispatcher_clone.dispatch(action));
        });

        Self { play_button, current_song_info, state }
    }

    fn toggle_image(&self) {
        let is_playing = self.state.borrow().is_playing();
        self.play_button.get_children().first()
            .and_then(|child| child.downcast_ref::<gtk::Image>())
            .map(|image| {
                let new_image_name = if is_playing {
                    "media-playback-pause"
                } else {
                    "media-playback-start"
                };
                image.set_from_icon_name(Some(new_image_name), gtk::IconSize::Button);
            })
            .expect("error updating icon");
    }

    fn update_current_info(&self) {
        let state = self.state.borrow();
        if let Some(song) = state.current_song() {
            let label = format!("<b>{}</b>\n{}", &song.title, &song.artist);
            self.current_song_info.set_label(&label[..]);
        }
    }
}

impl<State> Component for Playback<State> where State: PlaybackState {

    fn handle(&self, action: AppAction) {
        match action {
            AppAction::Play|AppAction::Pause => {
                self.toggle_image();
            },
            AppAction::Load(_) => {
                self.update_current_info();
                self.toggle_image();
            },
            _ => {}
        }
    }
}
