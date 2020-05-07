use futures::sync::mpsc::Receiver;
use futures::stream::Stream;
use futures::future::{Future, ok};

use tokio_core::reactor::{Core, Handle};

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::SpotifyId;

use librespot::playback::config::PlayerConfig;
use librespot::playback::audio_backend;
use librespot::playback::player::Player;

pub enum PlayerAction {
    Login(String, String),
    Load(SpotifyId),
    Play,
    Pause
}

pub struct SpotifyPlayer {
    player: Option<Player>
}

impl SpotifyPlayer {

    pub fn create_player(&self, username: String, password: String, handle: Handle) -> Box<dyn Future<Item=Player, Error=()>> {

        let player_config = PlayerConfig::default();
        let session_config = SessionConfig::default();

        let credentials = Credentials::with_password(username, password);

        let player = Session::connect(session_config, credentials, None, handle).map(|session| {
            let backend = audio_backend::find(None).unwrap();
            let (player, _) = Player::new(player_config, session, None, move || {
                backend(None)
            });
            player
        });

        return Box::new(player.map_err(|_| {
            println!("error here");
            ()
        }))
    }

    pub fn new() -> Self {
        Self { player: None }
    }

    pub fn start(&mut self, handle: Handle, receiver: Receiver<PlayerAction>) -> impl Future + '_ {
         receiver.for_each(move |action| {
            let mut player = self.player.as_ref();
            match action {
                PlayerAction::Play => {
                    if let Some(ref player) = player {
                        player.play();
                    }
                },
                PlayerAction::Pause => {
                    if let Some(ref player) = player {
                        player.play();
                    }
                },
                PlayerAction::Load(track) => {
                    if let Some(ref player) = player {
                        handle
                            .spawn(player.load(track, true, 0)
                            .map_err(|_| ()));
                    }
                },
                PlayerAction::Login(username, password) => {
                    let created_player = self.create_player(username, password, handle.clone())
                        .wait().unwrap();
                    player.replace(&created_player);
                }
            };
            ok(())
        })
    }
}
