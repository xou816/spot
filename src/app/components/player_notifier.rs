use futures::channel::mpsc::Sender;
use librespot::core::spotify_id::SpotifyId;

use crate::app::backend::Command;
use crate::app::components::EventListener;
use crate::app::AppEvent;

pub struct PlayerNotifier {
    sender: Sender<Command>,
}

impl PlayerNotifier {
    pub fn new(sender: Sender<Command>) -> Self {
        Self { sender }
    }
}

impl EventListener for PlayerNotifier {
    fn on_event(&mut self, event: &AppEvent) {
        let command = match event {
            AppEvent::TrackPaused => Some(Command::PlayerPause),
            AppEvent::TrackResumed => Some(Command::PlayerResume),
            AppEvent::TrackChanged(uri) => SpotifyId::from_uri(&uri)
                .ok()
                .map(Command::PlayerLoad),
            AppEvent::LoginStarted(username, password) => {
                Some(Command::Login(username.to_owned(), password.to_owned()))
            }
            AppEvent::TrackSeeked(position) => Some(Command::PlayerSeek(*position)),
            _ => None,
        };

        if let Some(command) = command {
            let _ = self.sender.clone().try_send(command);
        }
    }
}
