use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;
use futures::future::{Future, FutureExt, TryFutureExt};
use futures::compat::Future01CompatExt;
use futures01::future::Future as OldFuture;

use tokio_core::reactor::{Handle};

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::SpotifyId;

use librespot::playback::config::PlayerConfig;
use librespot::playback::audio_backend;
use librespot::playback::player::Player;

use std::rc::Rc;
use std::cell::RefCell;

pub enum PlayerAction {
    Login(String, String),
    Load(SpotifyId),
    Play,
    Pause
}

pub struct SpotifyPlayer {
    player: Rc<RefCell<Option<Player>>>
}

impl SpotifyPlayer {

    pub fn new() -> Self {
        Self { player: Rc::new(RefCell::new(None)) }
    }

    pub async fn start(&self, handle: Handle, receiver: Receiver<PlayerAction>) -> Result<(), ()> {
        receiver.for_each(|action| {
            async {
                match action {
                    PlayerAction::Play => {
                        self.player.borrow().as_ref().map(|p| {
                            p.play();
                        });
                    },
                    PlayerAction::Pause => {
                        self.player.borrow().as_ref().map(|p| {
                            p.pause();
                        });
                    },
                    PlayerAction::Load(track) => {
                        self.player.borrow().as_ref().map(|p| {
                            handle.spawn(p.load(track, true, 0).map_err(|_| ()));
                        });
                    },
                    PlayerAction::Login(username, password) => {
                        if let Some(player) = create_player(username, password, handle.clone()).await {
                            self.player.borrow_mut().replace(player);
                        }
                    }
                }
            }
        }).await;
        Ok(())
    }
}


async fn create_session(username: String, password: String, handle: Handle) -> Option<Session> {
    let session_config = SessionConfig::default();
    let credentials = Credentials::with_password(username, password);
    let result = Session::connect(session_config, credentials, None, handle).compat().await;
    result.ok()
}

async fn create_player(username: String, password: String, handle: Handle) -> Option<Player> {
    if let Some(session) = create_session(username, password, handle).await {
        let backend = audio_backend::find(None).unwrap();
        let player_config = PlayerConfig::default();
        let (new_player, _) = Player::new(player_config, session, None, move || backend(None));
        Some(new_player)
    } else {
        None
    }
}
