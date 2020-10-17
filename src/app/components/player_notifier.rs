use futures::channel::mpsc::Sender;
use librespot::core::spotify_id::SpotifyId;

use crate::app::AppEvent;
use crate::app::backend::Command;
use crate::app::components::Component;


pub struct PlayerNotifier {
    sender: Sender<Command>
}

impl PlayerNotifier {
    pub fn new(sender: Sender<Command>) -> Self {
        Self { sender }
    }
}

impl Component for PlayerNotifier {

    fn on_event(&self, event: AppEvent) {

        let command = match event {
            AppEvent::TrackPaused => Some(Command::PlayerPause),
            AppEvent::TrackResumed => Some(Command::PlayerResume),
            AppEvent::TrackChanged(uri) => {
                SpotifyId::from_uri(&uri).ok().map(|uri| Command::PlayerLoad(uri))
            },
            AppEvent::LoginStarted(username, password) => Some(Command::Login(username, password)),
            AppEvent::TrackSeeked(position) => Some(Command::PlayerSeek(position)),
            _ => None
        };

        if let Some(command) = command {
            self.sender.clone().try_send(command).expect("Could not communicate with backend")
        }
    }

}
