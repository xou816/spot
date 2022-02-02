use std::ops::Deref;
use std::rc::Rc;

use futures::channel::mpsc::UnboundedSender;
use librespot::core::spotify_id::SpotifyId;

use crate::api::SpotifyConnectPlayer;
use crate::app::components::EventListener;
use crate::app::state::{
    Device, LoginAction, LoginEvent, LoginStartedEvent, PlaybackAction, PlaybackEvent,
};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, SongsSource};
use crate::player::Command;

enum CurrentlyPlaying {
    WithSource {
        context: String,
        offset: usize,
        song: String,
    },
    Song(String),
}

impl CurrentlyPlaying {
    fn song_id(&self) -> &String {
        match self {
            Self::WithSource { song, .. } => song,
            Self::Song(s) => s,
        }
    }
}

pub struct PlayerNotifier {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
    command_sender: UnboundedSender<Command>,
}

impl PlayerNotifier {
    pub fn new(
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        command_sender: UnboundedSender<Command>,
    ) -> Self {
        Self {
            app_model,
            dispatcher,
            command_sender,
        }
    }

    fn currently_playing(&self) -> Option<CurrentlyPlaying> {
        let state = self.app_model.get_state();
        let song = state.playback.current_song_id()?;
        let offset = state.playback.current_song_index();
        let context = state
            .playback
            .current_source()
            .and_then(SongsSource::spotify_uri);
        Some(if let (Some(offset), Some(context)) = (offset, context) {
            CurrentlyPlaying::WithSource {
                context,
                offset,
                song,
            }
        } else {
            CurrentlyPlaying::Song(song)
        })
    }

    fn device(&self) -> impl Deref<Target = Device> + '_ {
        self.app_model.map_state(|s| s.playback.current_device())
    }

    fn player(&self) -> SpotifyConnectPlayer {
        SpotifyConnectPlayer::new(self.app_model.get_spotify())
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
        let player = self.player();
        let event = event.clone();
        let currently_playing = self.currently_playing();
        self.dispatcher.dispatch_async(Box::pin(async move {
            match event {
                PlaybackEvent::TrackChanged(_) | PlaybackEvent::SourceChanged => {
                    match currently_playing {
                        Some(CurrentlyPlaying::WithSource {
                            context, offset, ..
                        }) => player.play_in_context(context, offset).await.ok()?,
                        Some(CurrentlyPlaying::Song(id)) => {
                            player.play(format!("spotify:track:{}", id)).await.ok()?
                        }
                        None => {}
                    }
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

    fn switch_device(&mut self, device: &Device) {
        match device {
            Device::Connect(_) => {
                self.send_command_to_local_player(Command::PlayerStop);
                self.notify_connect_player(&PlaybackEvent::SourceChanged);
                self.dispatcher.dispatch(PlaybackAction::Seek(0).into());
            }
            Device::Local => {
                let id = self
                    .currently_playing()
                    .and_then(|c| SpotifyId::from_base62(c.song_id()).ok());
                if let Some(id) = id {
                    self.notify_connect_player(&PlaybackEvent::PlaybackPaused);
                    self.send_command_to_local_player(Command::PlayerLoad(id));
                    self.send_command_to_local_player(Command::PlayerResume);
                }
            }
        }
    }
}

impl EventListener for PlayerNotifier {
    fn on_event(&mut self, event: &AppEvent) {
        let device = self.device().clone();
        match (device, event) {
            (_, AppEvent::LoginEvent(event)) => self.notify_login(event),
            (_, AppEvent::PlaybackEvent(PlaybackEvent::SwitchedDevice(d))) => self.switch_device(d),
            (Device::Local, AppEvent::PlaybackEvent(event)) => self.notify_local_player(event),
            (Device::Local, AppEvent::SettingsEvent(SettingsEvent::PlayerSettingsChanged) => self.send_command_to_local_player(Command::ReloadSettings),
            (Device::Connect(_), AppEvent::PlaybackEvent(event)) => {
                self.notify_connect_player(event)
            }
            _ => {}
        }
    }
}
