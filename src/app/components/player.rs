use futures::channel::mpsc::Sender;
use std::rc::Rc;
use librespot::core::spotify_id::SpotifyId;

use crate::app::AppAction;
use crate::app::backend::Command;
use crate::app::components::Component;

pub trait PlayerModel {
    fn current_song_uri(&self) -> Option<String>;
}

pub struct Player {
    model: Rc<dyn PlayerModel>,
    sender: Sender<Command>
}

impl Player {
    pub fn new(sender: Sender<Command>, model: Rc<dyn PlayerModel>) -> Self {
        Self { sender, model }
    }
}

impl Component for Player {
    fn handle(&self, message: &AppAction) {
        let mut sender = self.sender.clone();

        let option = match message.clone() {
            AppAction::Play => sender.try_send(Command::PlayerResume).ok(),
            AppAction::Pause => sender.try_send(Command::PlayerPause).ok(),
            AppAction::Load(track) => {
                SpotifyId::from_uri(&track).ok()
                    .and_then(|id| {
                        sender.try_send(Command::PlayerLoad(id)).ok()
                    })
            },
            AppAction::Next|AppAction::Previous => {
                self.model
                    .current_song_uri()
                    .and_then(|uri| SpotifyId::from_uri(&uri).ok())
                    .and_then(|id| {
                        sender.try_send(Command::PlayerLoad(id)).ok()
                    })
            },
            AppAction::TryLogin(username, password) => {
                sender.try_send(Command::Login(username, password)).ok()
            },
            AppAction::Seek(position) => sender.try_send(Command::PlayerSeek(position)).ok(),
            _ => Some(())
        };
        option.expect("Could not communicate with backend");
    }
}
