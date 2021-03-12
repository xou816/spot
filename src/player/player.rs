use futures::channel::mpsc::UnboundedReceiver;
use futures::compat::Future01CompatExt;
use futures::stream::StreamExt;
use futures01::future::Future as OldFuture;
use futures01::stream::Stream as OldStream;

use tokio_core::reactor::Handle;

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::keymaster;
use librespot::core::session::Session;

use librespot::playback::audio_backend;
use librespot::playback::config::{Bitrate, PlayerConfig};
use librespot::playback::player::{Player, PlayerEvent, PlayerEventChannel};

use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::{Rc, Weak};

use super::Command;
use crate::app::credentials;

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
    fn login_successful(&self, credentials: credentials::Credentials);
    fn refresh_successful(&self, token: String);
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
    settings: SpotifyPlayerSettings,
    player: RefCell<Option<Player>>,
    session: RefCell<Option<Session>>,
    delegate: Rc<dyn SpotifyPlayerDelegate>,
}

impl SpotifyPlayer {
    pub fn new(settings: SpotifyPlayerSettings, delegate: Rc<dyn SpotifyPlayerDelegate>) -> Self {
        Self {
            settings,
            player: RefCell::new(None),
            session: RefCell::new(None),
            delegate,
        }
    }

    async fn handle(&self, action: Command, handle: &Handle) -> Result<(), SpotifyError> {
        let mut player = self.player.borrow_mut();
        let mut session = self.session.borrow_mut();
        match action {
            Command::PlayerResume => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.play();
                Ok(())
            }
            Command::PlayerPause => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.pause();
                Ok(())
            }
            Command::PlayerStop => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.stop();
                Ok(())
            }
            Command::PlayerSeek(position) => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                player.seek(position);
                Ok(())
            }
            Command::PlayerLoad(track) => {
                let player = player.as_mut().ok_or(SpotifyError::PlayerNotReady)?;
                player.load(track, true, 0);
                Ok(())
            }
            Command::RefreshToken => {
                let session = session.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                let token = get_access_token(&session).await?;
                self.delegate.refresh_successful(token);
                Ok(())
            }
            Command::Logout => {
                session
                    .take()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .shutdown();
                let _ = player.take();
                Ok(())
            }
            Command::Login(username, password) => {
                let new_session =
                    create_session(username.clone(), password.clone(), handle.clone()).await?;
                let token = get_access_token(&new_session).await?;
                let credentials = credentials::Credentials {
                    username,
                    password,
                    token,
                    country: new_session.country(),
                };
                self.delegate.login_successful(credentials);

                let (new_player, channel) = self.create_player(new_session.clone());
                handle.spawn(player_setup_delegate(
                    channel,
                    Rc::downgrade(&self.delegate),
                ));
                player.replace(new_player);
                session.replace(new_session);

                Ok(())
            }
        }
    }

    fn create_player(&self, session: Session) -> (Player, PlayerEventChannel) {
        let backend = self.settings.backend.clone();

        let mut player_config = PlayerConfig::default();
        player_config.bitrate = self.settings.bitrate;
        println!("bitrate: {:?}", &player_config.bitrate);

        Player::new(player_config, session, None, move || match backend {
            AudioBackend::PulseAudio => {
                println!("using pulseaudio");
                let backend = audio_backend::find(Some("pulseaudio".to_string())).unwrap();
                backend(None)
            }
            AudioBackend::Alsa(device) => {
                println!("using alsa ({})", &device);
                let backend = audio_backend::find(Some("alsa".to_string())).unwrap();
                backend(Some(device.to_string()))
            }
        })
    }

    pub async fn start(
        self,
        handle: Handle,
        receiver: UnboundedReceiver<Command>,
    ) -> Result<(), ()> {
        let _self = &self;
        let handle = &handle;
        receiver
            .for_each(|action| async move {
                match _self.handle(action, handle).await {
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
user-read-recently-played";

async fn get_access_token(session: &Session) -> Result<String, SpotifyError> {
    let token = keymaster::get_token(session, CLIENT_ID, SCOPES)
        .compat()
        .await;
    token
        .map(|t| t.access_token)
        .map_err(|_| SpotifyError::TokenFailed)
}

async fn create_session(
    username: String,
    password: String,
    handle: Handle,
) -> Result<Session, SpotifyError> {
    let session_config = SessionConfig::default();
    let credentials = Credentials::with_password(username, password);
    let result = Session::connect(session_config, credentials, None, handle)
        .compat()
        .await;
    result.map_err(|_| SpotifyError::LoginFailed)
}

fn player_setup_delegate(
    channel: PlayerEventChannel,
    delegate: Weak<dyn SpotifyPlayerDelegate>,
) -> impl OldFuture<Item = (), Error = ()> {
    channel.for_each(move |event| {
        let delegate = delegate.upgrade().ok_or(())?;
        match event {
            PlayerEvent::EndOfTrack { .. } | PlayerEvent::Stopped { .. } => {
                delegate.end_of_track_reached();
            }
            PlayerEvent::Playing { position_ms, .. } => {
                delegate.notify_playback_state(position_ms);
            }
            _ => {}
        }
        Ok(())
    })
}
