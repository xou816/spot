use futures::channel::mpsc::Receiver;
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
use librespot::playback::config::PlayerConfig;
use librespot::playback::player::{Player, PlayerEvent};

use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fmt;
use std::rc::{Rc, Weak};

use super::Command;
use crate::app::credentials;

#[derive(Debug)]
pub enum SpotifyError {
    LoginFailed,
    PlayerNotReady,
}

impl Error for SpotifyError {}

impl fmt::Display for SpotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoginFailed => write!(f, "LoginFailed"),
            Self::PlayerNotReady => write!(f, "PlayerNotReady"),
        }
    }
}

pub trait SpotifyPlayerDelegate {
    fn end_of_track_reached(&self);
    fn login_successful(&self, credentials: credentials::Credentials);
    fn report_error(&self, error: SpotifyError);
    fn notify_playback_state(&self, position: u32);
}

pub struct SpotifyPlayer {
    player: RefCell<Option<Player>>,
    delegate: Rc<dyn SpotifyPlayerDelegate>,
}

impl SpotifyPlayer {
    pub fn new(delegate: Rc<dyn SpotifyPlayerDelegate>) -> Self {
        Self {
            player: RefCell::new(None),
            delegate,
        }
    }

    async fn handle(&self, action: Command, handle: &Handle) -> Result<(), SpotifyError> {
        let mut player = self.player.borrow_mut();
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
            Command::Login(username, password) => {
                let session =
                    create_session(username.clone(), password.clone(), handle.clone()).await?;
                let token = get_access_token(&session).await?;
                let credentials = credentials::Credentials {
                    username,
                    password,
                    token,
                    country: session.country(),
                };
                self.delegate.login_successful(credentials);

                let new_player = create_player(session);
                handle.spawn(player_subscribe_to_playing_event(
                    &new_player,
                    Rc::downgrade(&self.delegate),
                ));
                handle.spawn(player_end_of_track_event(
                    &new_player,
                    Rc::downgrade(&self.delegate),
                ));
                player.replace(new_player);

                Ok(())
            }
        }
    }

    pub async fn start(self, handle: Handle, receiver: Receiver<Command>) -> Result<(), ()> {
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
        .map_err(|_| SpotifyError::LoginFailed)
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

fn create_player(session: Session) -> Player {
    let backend = audio_backend::find(None).unwrap();
    let player_config = PlayerConfig::default();
    let (new_player, _) = Player::new(player_config, session, None, move || backend(None));
    new_player
}

fn player_end_of_track_event(
    player: &Player,
    delegate: Weak<dyn SpotifyPlayerDelegate>,
) -> impl OldFuture<Item = (), Error = ()> {
    player
        .get_player_event_channel()
        .filter(|event| match event {
            PlayerEvent::EndOfTrack { .. } | PlayerEvent::Stopped { .. } => true,
            _ => false,
        })
        .for_each(move |_| {
            delegate.upgrade().ok_or(())?.end_of_track_reached();
            Ok(())
        })
}

fn player_subscribe_to_playing_event(
    player: &Player,
    delegate: Weak<dyn SpotifyPlayerDelegate>,
) -> impl OldFuture<Item = (), Error = ()> {
    player
        .get_player_event_channel()
        .filter_map(|event| match event {
            PlayerEvent::Playing { position_ms, .. } => Some(position_ms),
            _ => None,
        })
        .for_each(move |position_ms| {
            delegate
                .upgrade()
                .ok_or(())?
                .notify_playback_state(position_ms);
            Ok(())
        })
}
