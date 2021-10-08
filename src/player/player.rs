use futures::channel::mpsc::UnboundedReceiver;
use futures::stream::StreamExt;

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::keymaster;
use librespot::core::session::Session;

use librespot::protocol::authentication::AuthenticationType;

use librespot::playback::audio_backend;
use librespot::playback::config::{AudioFormat, Bitrate, PlayerConfig};
use librespot::playback::player::{Player, PlayerEvent, PlayerEventChannel};

use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use super::Command;
use crate::api::SpotifyApiClient;
use crate::app::credentials;
use crate::app::state::Device;

#[derive(Debug)]
pub enum SpotifyError {
    LoginFailed,
    TokenFailed,
    PlayerNotReady,
}

impl Error for SpotifyError {}

impl fmt::Display for SpotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoginFailed => write!(f, "Login failed!"),
            Self::TokenFailed => write!(f, "Token retrieval failed!"),
            Self::PlayerNotReady => write!(f, "Player is not responding."),
        }
    }
}

pub trait SpotifyPlayerDelegate {
    fn end_of_track_reached(&self);
    fn password_login_successful(&self, credentials: credentials::Credentials);
    fn token_login_successful(&self, username: String, token: String);
    fn refresh_successful(&self, token: String, token_expiry_time: SystemTime);
    fn report_error(&self, error: SpotifyError);
    fn notify_playback_state(&self, position: u32);
}

#[derive(Clone)]
pub enum AudioBackend {
    PulseAudio,
    Alsa(String),
}

#[derive(Clone)]
pub struct SpotifyPlayerSettings {
    pub bitrate: Bitrate,
    pub backend: AudioBackend,
}

impl Default for SpotifyPlayerSettings {
    fn default() -> Self {
        Self {
            bitrate: Bitrate::Bitrate160,
            backend: AudioBackend::PulseAudio,
        }
    }
}

pub struct SpotifyPlayer {
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
    settings: SpotifyPlayerSettings,
    player: RefCell<Option<Player>>,
    session: RefCell<Option<Session>>,
    delegate: Rc<dyn SpotifyPlayerDelegate>,
    device: RefCell<Device>,
}

impl SpotifyPlayer {
    pub fn new(
        api: Arc<dyn SpotifyApiClient + Send + Sync>,
        settings: SpotifyPlayerSettings,
        delegate: Rc<dyn SpotifyPlayerDelegate>,
    ) -> Self {
        Self {
            api,
            settings,
            player: RefCell::new(None),
            session: RefCell::new(None),
            delegate,
            device: RefCell::new(Device::Connect),
        }
    }

    async fn handle(&self, action: Command) -> Result<(), SpotifyError> {
        let mut player = self.player.borrow_mut();
        let mut session = self.session.borrow_mut();
        match (*self.device.borrow(), action) {
            (Device::Local, Command::PlayerResume) => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.play();
                Ok(())
            }
            (Device::Connect, Command::PlayerResume) => {
                self.api.player_play(None).await.unwrap();
                Ok(())
            }
            (Device::Local, Command::PlayerPause) => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.pause();
                self.api.player_pause().await.unwrap();
                Ok(())
            }
            (Device::Connect, Command::PlayerPause) => {
                self.api.player_pause().await.unwrap();
                Ok(())
            }
            (Device::Local, Command::PlayerStop) => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.stop();
                Ok(())
            }
            (Device::Connect, Command::PlayerStop) => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.stop();
                Ok(())
            }
            (Device::Local, Command::PlayerSeek(position)) => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.seek(position);
                Ok(())
            }
            (Device::Connect, Command::PlayerSeek(position)) => {
                self.api.player_seek(position as usize).await.unwrap();
                Ok(())
            }
            (Device::Local, Command::PlayerLoad(track)) => {
                let player = player.as_mut().ok_or(SpotifyError::PlayerNotReady)?;
                player.load(track, true, 0);
                Ok(())
            }
            (Device::Connect, Command::PlayerLoad(track)) => {
                let uri = track.to_uri();
                self.api.player_play(Some(uri)).await.unwrap();
                Ok(())
            }
            (Device::Local, Command::SwitchDevice(Device::Local)) => Ok(()),
            (Device::Local, Command::SwitchDevice(Device::Connect)) => {
                let player = player.as_mut().ok_or(SpotifyError::PlayerNotReady)?;
                player.pause();
                *self.device.borrow_mut() = Device::Connect;
                self.api.player_play(None).await.unwrap();
                Ok(())
            }
            (Device::Connect, Command::SwitchDevice(Device::Local)) => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                self.api.player_pause().await.unwrap();
                *self.device.borrow_mut() = Device::Local;
                player.play();
                Ok(())
            }
            (Device::Connect, Command::SwitchDevice(Device::Connect)) => Ok(()),
            (_, Command::RefreshToken) => {
                let session = session.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                let (token, token_expiry_time) = get_access_token_and_expiry_time(session).await?;
                self.delegate.refresh_successful(token, token_expiry_time);
                Ok(())
            }
            (_, Command::Logout) => {
                session
                    .take()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .shutdown();
                let _ = player.take();
                Ok(())
            }
            (_, Command::PasswordLogin { username, password }) => {
                let credentials = Credentials::with_password(username, password.clone());
                let new_session = create_session(credentials).await?;
                let (token, token_expiry_time) =
                    get_access_token_and_expiry_time(&new_session).await?;
                let credentials = credentials::Credentials {
                    username: new_session.username(),
                    password,
                    token,
                    token_expiry_time: Some(token_expiry_time),
                    country: new_session.country(),
                };
                self.delegate.password_login_successful(credentials);

                let (new_player, channel) = self.create_player(new_session.clone());
                tokio::task::spawn_local(player_setup_delegate(channel, Rc::clone(&self.delegate)));
                player.replace(new_player);
                session.replace(new_session);

                Ok(())
            }
            (_, Command::TokenLogin { username, token }) => {
                let credentials = Credentials {
                    username,
                    auth_type: AuthenticationType::AUTHENTICATION_SPOTIFY_TOKEN,
                    auth_data: token.clone().into_bytes(),
                };
                let new_session = create_session(credentials).await?;
                self.delegate
                    .token_login_successful(new_session.username(), token);

                let (new_player, channel) = self.create_player(new_session.clone());
                tokio::task::spawn_local(player_setup_delegate(channel, Rc::clone(&self.delegate)));
                player.replace(new_player);
                session.replace(new_session);

                Ok(())
            }
        }
    }

    fn create_player(&self, session: Session) -> (Player, PlayerEventChannel) {
        let backend = self.settings.backend.clone();

        let player_config = PlayerConfig {
            bitrate: self.settings.bitrate,
            ..Default::default()
        };
        println!("bitrate: {:?}", &player_config.bitrate);

        Player::new(player_config, session, None, move || match backend {
            AudioBackend::PulseAudio => {
                println!("using pulseaudio");
                let backend = audio_backend::find(Some("pulseaudio".to_string())).unwrap();
                backend(None, AudioFormat::default())
            }
            AudioBackend::Alsa(device) => {
                println!("using alsa ({})", &device);
                let backend = audio_backend::find(Some("alsa".to_string())).unwrap();
                backend(Some(device), AudioFormat::default())
            }
        })
    }

    pub async fn start(self, receiver: UnboundedReceiver<Command>) -> Result<(), ()> {
        let _self = &self;
        receiver
            .for_each(|action| async move {
                match _self.handle(action).await {
                    Ok(_) => {}
                    Err(err) => _self.delegate.report_error(err),
                }
            })
            .await;
        Ok(())
    }
}

const CLIENT_ID: &str = "782ae96ea60f4cdf986a766049607005";

const SCOPES: &str = "user-read-private,\
playlist-read-private,\
playlist-read-collaborative,\
user-library-read,\
user-library-modify,\
user-top-read,\
user-read-recently-played,\
playlist-modify-public,\
playlist-modify-private,\
user-modify-playback-state,\
streaming";

async fn get_access_token_and_expiry_time(
    session: &Session,
) -> Result<(String, SystemTime), SpotifyError> {
    let token = keymaster::get_token(session, CLIENT_ID, SCOPES)
        .await
        .map_err(|e| {
            dbg!(e);
            SpotifyError::TokenFailed
        })?;
    let expiry_time = SystemTime::now() + Duration::from_secs(token.expires_in.into());
    Ok((token.access_token, expiry_time))
}

async fn create_session(credentials: Credentials) -> Result<Session, SpotifyError> {
    let session_config = SessionConfig::default();
    let result = Session::connect(session_config, credentials, None).await;
    result.map_err(|e| {
        dbg!(e);
        SpotifyError::LoginFailed
    })
}

async fn player_setup_delegate(
    mut channel: PlayerEventChannel,
    delegate: Rc<dyn SpotifyPlayerDelegate>,
) {
    while let Some(event) = channel.recv().await {
        match event {
            PlayerEvent::EndOfTrack { .. } | PlayerEvent::Stopped { .. } => {
                delegate.end_of_track_reached();
            }
            PlayerEvent::Playing { position_ms, .. } => {
                delegate.notify_playback_state(position_ms);
            }
            _ => {}
        }
    }
}
