use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender;
use librespot::core::spotify_id::SpotifyId;

use crate::api::{SpotifyApiClient, SpotifyConnectPlayer};
use crate::app::components::EventListener;
use crate::app::models::ConnectDevice;
use crate::app::state::{Device, LoginAction, LoginEvent, LoginStartedEvent, PlaybackEvent};
use crate::app::{ActionDispatcher, AppAction, AppEvent};
use crate::player::Command;

pub struct PlayerNotifier {
    device: Device,
    connect_player: SpotifyConnectPlayer,
    dispatcher: Box<dyn ActionDispatcher>,
    command_sender: UnboundedSender<Command>,
}

impl PlayerNotifier {
    pub fn new(
        api: Arc<dyn SpotifyApiClient + Send + Sync>,
        dispatcher: Box<dyn ActionDispatcher>,
        command_sender: UnboundedSender<Command>,
    ) -> Self {
        Self {
            device: Device::Connect(ConnectDevice {
                id: "cafecafe".to_string(),
                label: "My device".to_string(),
            }),
            connect_player: SpotifyConnectPlayer::new(api),
            dispatcher,
            command_sender,
        }
    }

    fn notify_login(&self, event: &LoginEvent) {
        let command = match event {
            LoginEvent::LoginStarted(LoginStartedEvent::Password { username, password }) => {
                Some(Command::PasswordLogin {
                    username: username.to_owned(),
                    password: password.to_owned(),
                })
            }
            LoginEvent::LoginStarted(LoginStartedEvent::Token { username, token }) => {
                Some(Command::TokenLogin {
                    username: username.to_owned(),
                    token: token.to_owned(),
                })
            }
            LoginEvent::FreshTokenRequested => Some(Command::RefreshToken),
            LoginEvent::LogoutCompleted => Some(Command::Logout),
            _ => None,
        };

        if let Some(command) = command {
            self.send_command_to_local_player(command);
        }
    }

    fn notify_connect_player(&self, event: &PlaybackEvent) {
        let player = self.connect_player.clone();
        let event = event.clone();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match event {
                PlaybackEvent::TrackChanged(id) => {
                    player.play(format!("spotify:track:{}", id)).await.ok()?
                }
                PlaybackEvent::TrackSeeked(position) => player.seek(position).await.ok()?,
                PlaybackEvent::PlaybackPaused => player.pause().await.ok()?,
                PlaybackEvent::PlaybackResumed => player.resume().await.ok()?,
                _ => {}
            };
            None
        }))
    }

    fn notify_local_player(&self, event: &PlaybackEvent) {
        let command = match event {
            PlaybackEvent::PlaybackPaused => Some(Command::PlayerPause),
            PlaybackEvent::PlaybackResumed => Some(Command::PlayerResume),
            PlaybackEvent::PlaybackStopped => Some(Command::PlayerStop),
            PlaybackEvent::VolumeSet(volume) => Some(Command::PlayerSetVolume(*volume)),
            PlaybackEvent::TrackChanged(id) => {
                SpotifyId::from_base62(id).ok().map(Command::PlayerLoad)
            }
            PlaybackEvent::TrackSeeked(position) => Some(Command::PlayerSeek(*position)),
            _ => None,
        };

        if let Some(command) = command {
            self.send_command_to_local_player(command);
        }
    }

    fn send_command_to_local_player(&self, command: Command) {
        let dispatcher = &self.dispatcher;
        self.command_sender
            .unbounded_send(command)
            .unwrap_or_else(|_| {
                dispatcher.dispatch(AppAction::LoginAction(LoginAction::SetLoginFailure));
            });
    }
}

impl EventListener for PlayerNotifier {
    fn on_event(&mut self, event: &AppEvent) {
        match (&self.device, event) {
            (_, AppEvent::LoginEvent(event)) => self.notify_login(event),
            (Device::Local, AppEvent::PlaybackEvent(event)) => self.notify_local_player(event),
            (Device::Local, AppEvent::SettingsEvent(SettingsEvent::PlayerSettingsChanged) => self.send_command_to_local_player(Command::ReloadSettings),
            (Device::Connect(_), AppEvent::PlaybackEvent(event)) => {
                self.notify_connect_player(event)
            }
            _ => {}
        }
    }
}
