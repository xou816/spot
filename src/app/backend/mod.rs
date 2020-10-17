use std::rc::Rc;
use std::thread;
use tokio_core::reactor::Core;
use futures::future::{FutureExt, TryFutureExt};
use futures::channel::mpsc::{Sender, channel};
use librespot::core::spotify_id::SpotifyId;

mod player;
pub use player::{SpotifyPlayer, SpotifyPlayerDelegate};

pub mod api;

use super::{Dispatcher, AppAction};
use crate::app::credentials;


#[derive(Debug, Clone)]
pub enum Command {
    Login(String, String),
    PlayerLoad(SpotifyId),
    PlayerResume,
    PlayerPause,
    PlayerSeek(u32)
}

struct AppPlayerDelegate {
    dispatcher: Dispatcher
}


impl AppPlayerDelegate {
    fn new(dispatcher: Dispatcher) -> Self {
        Self { dispatcher }
    }
}


impl SpotifyPlayerDelegate for AppPlayerDelegate {

    fn end_of_track_reached(&self) {
        self.dispatcher.dispatch(AppAction::Next).unwrap();
    }

    fn login_successful(&self, credentials: credentials::Credentials) {
        self.dispatcher.dispatch(AppAction::LoginSuccess(credentials));
    }
}

pub fn start_player_service(dispatcher: Dispatcher) -> Sender<Command> {
    let (sender, receiver) = channel::<Command>(0);
    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let delegate = Rc::new(AppPlayerDelegate::new(dispatcher));
        core.run(SpotifyPlayer::new(delegate)
            .start(core.handle(), receiver)
            .boxed_local()
            .compat())
            .expect("Player crashed!");
    });
    sender
}
