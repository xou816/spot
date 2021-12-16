use futures::channel::mpsc::UnboundedSender;
use librespot::core::spotify_id::SpotifyId;

use crate::app::components::EventListener;
use crate::app::state::{LoginAction, LoginEvent, LoginStartedEvent, PlaybackEvent};
use crate::app::{AppAction, AppEvent};
use crate::player::Command;

pub struct PlayerNotifier {
    action_sender: UnboundedSender<AppAction>,
    sender: UnboundedSender<Command>,
}

impl PlayerNotifier {
    pub fn new(
        action_sender: UnboundedSender<AppAction>,
        sender: UnboundedSender<Command>,
    ) -> Self {
        Self {
            action_sender,
            sender,
        }
    }
}

impl EventListener for PlayerNotifier {
    fn on_event(&mut self, event: &AppEvent) {
        let command = match event {
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackPaused) => Some(Command::PlayerPause),
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackResumed) => Some(Command::PlayerResume),
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackStopped) => Some(Command::PlayerStop),
            AppEvent::PlaybackEvent(PlaybackEvent::VolumeSet(volume)) => {
                Some(Command::PlayerVolume(*volume))
            }
            AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(id)) => {
                SpotifyId::from_base62(id).ok().map(Command::PlayerLoad)
            }
            AppEvent::PlaybackEvent(PlaybackEvent::TrackSeeked(position)) => {
                Some(Command::PlayerSeek(*position))
            }
            AppEvent::LoginEvent(LoginEvent::LoginStarted(LoginStartedEvent::Password {
                username,
                password,
            })) => Some(Command::PasswordLogin {
                username: username.to_owned(),
                password: password.to_owned(),
            }),
            AppEvent::LoginEvent(LoginEvent::LoginStarted(LoginStartedEvent::Token {
                username,
                token,
            })) => Some(Command::TokenLogin {
                username: username.to_owned(),
                token: token.to_owned(),
            }),
            AppEvent::LoginEvent(LoginEvent::FreshTokenRequested) => Some(Command::RefreshToken),
            AppEvent::LoginEvent(LoginEvent::LogoutCompleted) => Some(Command::Logout),
            _ => None,
        };

        if let Some(command) = command {
            let action_sender = &self.action_sender;
            self.sender.unbounded_send(command).unwrap_or_else(|_| {
                action_sender
                    .unbounded_send(AppAction::LoginAction(LoginAction::SetLoginFailure))
                    .unwrap();
            });
        }
    }
}
