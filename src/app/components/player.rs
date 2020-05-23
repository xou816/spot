use futures::channel::mpsc::Sender;
use librespot::core::spotify_id::SpotifyId;

use crate::app::AppAction;
use crate::app::backend::Command;
use crate::app::components::Component;

pub struct Player {
    sender: Sender<Command>
}

impl Player {
    pub fn new(sender: Sender<Command>) -> Self {
        Self { sender }
    }
}

impl Component for Player {
    fn handle(&self, message: &AppAction) {
        let mut sender = self.sender.clone();

        let option = match message.clone() {
            AppAction::Play => sender.try_send(Command::PlayerResume).ok(),
            AppAction::Pause => sender.try_send(Command::PlayerPause).ok(),
            AppAction::Load(track) => {
                if let Some(id) = SpotifyId::from_uri(&track).ok() {
                    sender.try_send(Command::PlayerLoad(id)).ok()
                } else {
                    None
                }
            },
            AppAction::TryLogin(username, password) => {
                sender.try_send(Command::Login(username, password)).ok()
            },
            _ => Some(())
        };
        option.expect("Could not communicate with backend");
    }
}
