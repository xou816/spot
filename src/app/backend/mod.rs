use std::thread;
use std::convert::Into;
use tokio_core::reactor::Core;
use futures::future::{FutureExt, TryFutureExt};
use futures::channel::mpsc::{Sender, channel};
use librespot::core::spotify_id::SpotifyId;

mod player;
pub use player::{SpotifyPlayer};

pub mod api;

use super::{Dispatcher, AppAction};
use crate::app::credentials;


#[derive(Debug, Clone)]
pub enum Command {
    Login(String, String),
    LoginSuccessful(credentials::Credentials),
    PlayerLoad(SpotifyId),
    PlayerResume,
    PlayerPause,
    PlayerSeek(u32),
    PlayerEndOfTrack
}

impl Into<Option<AppAction>> for Command {
    fn into(self) -> Option<AppAction> {
        match self {
            Command::LoginSuccessful(cred) => Some(AppAction::LoginSuccess(cred)),
            Command::PlayerEndOfTrack => Some(AppAction::Next),
            _ => None
        }
    }
}


pub fn start_player_service(dispatcher: Dispatcher) -> Sender<Command> {
    let (sender, receiver) = channel::<Command>(0);
    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        core.run(SpotifyPlayer::new(dispatcher)
            .start(core.handle(), receiver)
            .boxed_local()
            .compat())
            .expect("Player crashed!");
    });
    sender
}
