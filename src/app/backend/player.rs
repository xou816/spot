use futures::channel::mpsc::{Receiver};
use futures::stream::StreamExt;
use futures::compat::Future01CompatExt;
use futures01::future::Future as OldFuture;

use tokio_core::reactor::{Handle};

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::keymaster;

use librespot::playback::config::PlayerConfig;
use librespot::playback::audio_backend;
use librespot::playback::player::Player;

use std::rc::Rc;
use std::cell::RefCell;

use super::{Command, Dispatcher};
use crate::app::credentials;

pub struct SpotifyPlayer {
    player: Rc<RefCell<Option<Player>>>,
    sender: Dispatcher
}

impl SpotifyPlayer {

    pub fn new(sender: Dispatcher) -> Self {
        Self { player: Rc::new(RefCell::new(None)), sender }
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
                let sender = self.sender.clone();
                let player = player.as_ref().ok_or("Could not get player")?;
                let end_of_track = player.load(track, true, 0)
                    .map_err(|_| ())
                    .map(move |_| {
                        sender.dispatch(Command::PlayerEndOfTrack).unwrap();
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
                self.sender.clone().dispatch(Command::LoginSuccessful(credentials)).unwrap();
                player.replace(create_player(session));
                Ok(())
            },
            _ => Ok(())
        }
    }

    pub async fn start(&self, handle: Handle, receiver: Receiver<Command>) -> Result<(), ()> {
        receiver.for_each(|action| {
            async {
                self.handle(action, handle.clone()).await.unwrap();
            }
        }).await;
        Ok(())
    }
}

const CLIENT_ID: &'static str = "e1dce60f1e274e20861ce5d96142a4d3";

const SCOPES: &'static str = "user-read-private,\
playlist-read-private,\
playlist-read-collaborative,\
user-library-read,\
user-library-modify,\
user-top-read,\
user-read-recently-played";

async fn get_access_token(session: &Session) -> Result<String, &'static str> {
    let token = keymaster::get_token(&session, CLIENT_ID, SCOPES).compat().await;
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
