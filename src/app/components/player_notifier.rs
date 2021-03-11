use futures::channel::mpsc::UnboundedSender;
use librespot::core::spotify_id::SpotifyId;

use crate::app::backend::Command;
use crate::app::components::EventListener;
use crate::app::state::{LoginEvent, PlaybackEvent};
use crate::app::AppEvent;

pub struct PlayerNotifier {
    sender: UnboundedSender<Command>,
}

impl PlayerNotifier {
    pub fn new(sender: UnboundedSender<Command>) -> Self {
        Self { sender }
    }
}

impl EventListener for PlayerNotifier {
    fn on_event(&mut self, event: &AppEvent) {
        let command = match event {
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackPaused) => Some(Command::PlayerPause),
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackResumed) => Some(Command::PlayerResume),
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackStopped) => Some(Command::PlayerStop),
            AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(id)) => {
                SpotifyId::from_base62(&id).ok().map(Command::PlayerLoad)
            }
            AppEvent::PlaybackEvent(PlaybackEvent::TrackSeeked(position)) => {
                Some(Command::PlayerSeek(*position))
            }
            AppEvent::LoginEvent(LoginEvent::LoginStarted(username, password)) => {
                Some(Command::Login(username.to_owned(), password.to_owned()))
            }
            AppEvent::LoginEvent(LoginEvent::FreshTokenRequested) => Some(Command::RefreshToken),
            AppEvent::LoginEvent(LoginEvent::LogoutCompleted) => Some(Command::Logout),
            _ => None,
        };

        if let Some(command) = command {
            self.sender.unbounded_send(command).unwrap_or_else(|_| {
                println!("Could not send message to player");
            });
        }
    }
}
