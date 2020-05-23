use futures::channel::mpsc::{Receiver};
use futures::stream::StreamExt;
use futures::future::{FutureExt, TryFutureExt};
use futures::compat::Future01CompatExt;
use futures01::future::Future as OldFuture;

use tokio_core::reactor::{Handle};

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::SpotifyId;
use librespot::core::keymaster;

use librespot::playback::config::PlayerConfig;
use librespot::playback::audio_backend;
use librespot::playback::player::Player;

use std::rc::Rc;
use std::cell::RefCell;

use super::{Command, Dispatcher};

pub struct SpotifyPlayer {
    player: Rc<RefCell<Option<Player>>>,
    sender: Dispatcher
}

impl SpotifyPlayer {

    pub fn new(sender: Dispatcher) -> Self {
        Self { player: Rc::new(RefCell::new(None)), sender }
    }

    pub async fn start(&self, handle: Handle, receiver: Receiver<Command>) -> Result<(), ()> {
        receiver.for_each(|action| {
            async {
                match action {
                    Command::PlayerResume => {
                        self.player.borrow().as_ref().map(|p| {
                            p.play();
                        });
                    },
                    Command::PlayerPause => {
                        self.player.borrow().as_ref().map(|p| {
                            p.pause();
                        });
                    },
                    Command::PlayerLoad(track) => {
                        self.player.borrow().as_ref().map(|p| {
                            handle.spawn(p.load(track, true, 0).map_err(|_| ()));
                        });
                    },
                    Command::Login(username, password) => {
                        if let Some(session) = create_session(username, password, handle.clone()).await {

                            if let Some(token) = get_access_token(&session).await {
                                self.sender.clone().dispatch(Command::LoginSuccessful(token)).unwrap();
                            }

                            self.player.borrow_mut().replace(create_player(session));
                        }
                    },
                    _ => {}
                }
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

async fn get_access_token(session: &Session) -> Option<String> {
    let token = keymaster::get_token(&session, CLIENT_ID, SCOPES).compat().await;
    token.map(|t| t.access_token).ok()
}

async fn create_session(username: String, password: String, handle: Handle) -> Option<Session> {
    let session_config = SessionConfig::default();
    let credentials = Credentials::with_password(username, password);
    let result = Session::connect(session_config, credentials, None, handle).compat().await;
    result.ok()
}

fn create_player(session: Session) -> Player {
    let backend = audio_backend::find(None).unwrap();
    let player_config = PlayerConfig::default();
    let (new_player, _) = Player::new(player_config, session, None, move || backend(None));
    new_player
}
