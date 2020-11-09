use std::rc::Rc;
use std::thread;
use tokio_core::reactor::Core;
use futures::future::{FutureExt, TryFutureExt};
use futures::channel::mpsc::{Sender, channel};
use librespot::core::spotify_id::SpotifyId;

use super::{AppAction};
use crate::app::credentials;

mod player;
pub use player::{SpotifyPlayer, SpotifyPlayerDelegate};

pub mod api;
pub mod api_models;

pub mod cache;



#[derive(Debug, Clone)]
pub enum Command {
    Login(String, String),
    PlayerLoad(SpotifyId),
    PlayerResume,
    PlayerPause,
    PlayerSeek(u32)
}

struct AppPlayerDelegate {
    sender: Sender<AppAction>
}


impl AppPlayerDelegate {
    fn new(sender: Sender<AppAction>) -> Self {
        Self { sender }
    }
}


impl SpotifyPlayerDelegate for AppPlayerDelegate {

    fn end_of_track_reached(&self) {
        self.sender.clone().try_send(AppAction::Next).unwrap();
    }

    fn login_successful(&self, credentials: credentials::Credentials) {
        self.sender.clone().try_send(AppAction::LoginSuccess(credentials)).unwrap();
    }

    fn report_error(&self, error: &'static str) {
        println!("{}", error);
    }
}

pub fn start_player_service(appaction_sender: Sender<AppAction>) -> Sender<Command> {
    let (sender, receiver) = channel::<Command>(0);
    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let delegate = Rc::new(AppPlayerDelegate::new(appaction_sender));
        core.run(SpotifyPlayer::new(delegate)
            .start(core.handle(), receiver)
            .boxed_local()
            .compat())
            .expect("Player crashed!");
    });
    sender
}
