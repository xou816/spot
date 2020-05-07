use futures::sync::mpsc::Receiver;
use futures::stream::Stream;
use futures::future::{Future, result};

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

    pub fn create_player(&self, username: String, password: String, handle: Handle) -> Box<dyn Future<Item=(), Error=()>> {

        let player_config = PlayerConfig::default();
        let session_config = SessionConfig::default();

        let credentials = Credentials::with_password(username, password);

        let player_ref = Rc::clone(&self.player);

        Box::new(Session::connect(session_config, credentials, None, handle).map(move |session| {
            let backend = audio_backend::find(None).unwrap();
            let (new_player, _) = Player::new(player_config, session, None, move || {
                backend(None)
            });
            player_ref.replace(Some(new_player));
        }).map_err(|_| () ))
    }

    pub fn new() -> Self {
        Self { player: Rc::new(RefCell::new(None)) }
    }


    fn with_player<F, R>(&self, action: F) -> Box<dyn Future<Item=(), Error=()>> where F: Fn(&Player) -> R {
        Box::new(result(match self.player.borrow().as_ref() {
            Some(player) => { action(player); Ok(()) },
            None => Ok(())
        }))
    }

    pub fn start(&mut self, handle: Handle, receiver: Receiver<PlayerAction>) -> impl Future + '_ {
         receiver.for_each(move |action| {
            match action {
                PlayerAction::Play => {
                    self.with_player(|player| player.play())
                },
                PlayerAction::Pause => {
                    self.with_player(|player| player.pause())
                },
                PlayerAction::Load(track) => {
                    handle.spawn(self.with_player(|player| player.load(track, true, 0)));
                    sync_ok()
                },
                PlayerAction::Login(username, password) => {
                    self.create_player(username, password, handle.clone())
                },
                _ => sync_ok()
            }
        })
    }
}

fn sync_ok() -> Box<dyn Future<Item=(), Error=()>> {
    Box::new(result(Ok(())))
}
