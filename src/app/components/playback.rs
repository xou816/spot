use gtk::prelude::*;
use gtk::ToggleButton;
use std::rc::Rc;
use std::cell::RefCell;

use crate::app::AppAction;
use crate::app::components::{Component, Dispatcher};

pub trait PlaybackState {
    fn is_playing(&self) -> bool;
}

pub struct Playback<State: 'static> where State: PlaybackState {
    play_button: ToggleButton,
    state: Rc<RefCell<State>>
}

impl<State> Playback<State> where State: PlaybackState {

    pub fn new(
        builder: &gtk::Builder,
        state: Rc<RefCell<State>>,
        dispatcher: Dispatcher) -> Self {

        let play_button: gtk::ToggleButton = builder.get_object("play_pause").unwrap();

        let weak_state = Rc::downgrade(&state);

        play_button.connect_clicked(move |_| {
            weak_state.upgrade()
                .and_then(|state| {
                    let state = state.borrow();
                    dispatcher
                        .send(if state.is_playing() { AppAction::Pause } else { AppAction::Play })
                        .ok()
                });
        });

        Self { play_button, state }
    }
}

impl<State> Component for Playback<State> where State: PlaybackState {

    fn handle(&self, action: AppAction) {
        let should_update = match action {
            AppAction::Play|AppAction::Load(_)|AppAction::Pause => true,
            _ => false
        };

        if should_update {
            let state = self.state.borrow();
            let flags = if state.is_playing() {
                gtk::StateFlags::CHECKED
            } else {
                gtk::StateFlags::NORMAL
            };
            self.play_button.set_state_flags(flags, true);
        }
    }
}
