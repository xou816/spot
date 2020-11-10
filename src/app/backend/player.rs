use futures::channel::mpsc::{Receiver};
use futures::stream::StreamExt;
use futures::compat::Future01CompatExt;
use futures01::future::Future as OldFuture;
use futures01::stream::Stream as OldStream;

use tokio_core::reactor::Handle;

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::keymaster;

use librespot::playback::config::PlayerConfig;
use librespot::playback::audio_backend;
use librespot::playback::player::{Player, PlayerEvent};

use std::rc::{Rc, Weak};
use std::cell::RefCell;

use super::Command;
use crate::app::credentials;

pub trait SpotifyPlayerDelegate {
    fn end_of_track_reached(&self);
    fn login_successful(&self, credentials: credentials::Credentials);
    fn report_error(&self, error: &'static str);
    fn notify_playback_state(&self, position: u32);
}

pub struct SpotifyPlayer {
    player: RefCell<Option<Player>>,
    delegate: Rc<dyn SpotifyPlayerDelegate>
}

impl SpotifyPlayer {

    pub fn new(delegate: Rc<dyn SpotifyPlayerDelegate>) -> Self {
        Self { player: RefCell::new(None), delegate }
    }

    async fn handle(&self, action: Command, handle: Handle) -> Result<(), &'static str> {
        let mut player = self.player.borrow_mut();
        match action {
            Command::PlayerResume => {
                let player = player.as_ref().ok_or("Could not get player")?;
                player.play();
                Ok(())
            },
            Command::PlayerPause => {
                let player = player.as_ref().ok_or("Could not get player")?;
                player.pause();
                Ok(())
            },
            Command::PlayerSeek(position) => {
                let player = player.as_ref().ok_or("Could not get player")?;
                player.seek(position);
                Ok(())
            },
            Command::PlayerLoad(track) => {
                let delegate = Rc::downgrade(&self.delegate);
                let player = player.as_mut().ok_or("Could not get player")?;
                player.load(track, true, 0);
                let end_of_track = player.get_end_of_track_future()
                    .map(move |_| {
                        delegate.upgrade().map(|delegate| {
                            delegate.end_of_track_reached();
                        });
                    });
                handle.spawn(end_of_track);
                Ok(())
            },
            Command::Login(username, password) => {
                let session = create_session(username.clone(), password.clone(), handle.clone()).await?;
                let token = get_access_token(&session).await?.clone();
                let credentials = credentials::Credentials {
                    username, password, token: token.clone()
                };
                self.delegate.login_successful(credentials);

                let new_player = create_player(session);
                handle.spawn(player_subscribe_to_playing_event(
                    &new_player,
                    Rc::downgrade(&self.delegate)));
                player.replace(new_player);

                Ok(())
            }
        }
    }

    pub async fn start(&self, handle: Handle, receiver: Receiver<Command>) -> Result<(), ()> {
        receiver.for_each(|action| {
            async {
                match self.handle(action, handle.clone()).await {
                    Ok(_) => {},
                    Err(err) => self.delegate.report_error(err)
                }
            }
        }).await;
        Ok(())
    }
}

const CLIENT_ID: &'static str = "782ae96ea60f4cdf986a766049607005";

const SCOPES: &'static str = "user-read-private,\
playlist-read-private,\
playlist-read-collaborative,\
user-library-read,\
user-library-modify,\
user-top-read,\
user-read-recently-played";

async fn get_access_token(session: &Session) -> Result<String, &'static str> {
    let token = keymaster::get_token(session, CLIENT_ID, SCOPES).compat().await;
    token.map(|t| t.access_token).map_err(|_| "Error obtaining token")
}

async fn create_session(username: String, password: String, handle: Handle) -> Result<Session, &'static str> {
    let session_config = SessionConfig::default();
    let credentials = Credentials::with_password(username, password);
    let result = Session::connect(session_config, credentials, None, handle).compat().await;
    result.map_err(|_| "Error creating session")
}

fn create_player(session: Session) -> Player {
    let backend = audio_backend::find(None).unwrap();
    let player_config = PlayerConfig::default();
    let (new_player, _) = Player::new(player_config, session, None, move || backend(None));
    new_player
}

fn player_subscribe_to_playing_event(
    player: &Player,
    delegate: Weak<dyn SpotifyPlayerDelegate>) -> impl OldFuture<Item=(), Error=()> {

    player.get_player_event_channel()
        .filter_map(|event| {
            match event {
                PlayerEvent::Playing { position_ms, .. } => Some(position_ms),
                _ => None
            }
        })
        .for_each(move |position_ms| {
            delegate.upgrade().ok_or(())?.notify_playback_state(position_ms);
            Ok(())
        })
}
