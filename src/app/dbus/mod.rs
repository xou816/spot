use futures::channel::mpsc::Sender;
use std::convert::{TryInto};
use std::thread;
use zbus::fdo;

use crate::app::components::EventListener;
use crate::app::{AppAction, AppEvent};

mod mpris;
pub use mpris::*;

mod types;
use types::*;

pub struct ConnectionWrapper(zbus::ObjectServer);

impl ConnectionWrapper {
    fn new(connection: zbus::Connection, player: SpotMprisPlayer) -> Result<Self, zbus::Error> {
        let object_server = register_mpris(&connection, player)?;
        Ok(Self(object_server))
    }

    fn with_player<F: Fn(&SpotMprisPlayer) -> zbus::Result<()>>(&self, f: F) -> zbus::Result<()> {
        self.0.with(
            &"/org/mpris/MediaPlayer2".try_into()?,
            |iface: &SpotMprisPlayer| f(iface),
        )
    }
}

impl EventListener for ConnectionWrapper {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::TrackPaused => {
                self.with_player(|player| {
                    player.state.set_playing(false);
                    player.notify_playback_status()?;
                    Ok(())
                })
                .unwrap();
            }
            AppEvent::TrackResumed => {
                self.with_player(|player| {
                    player.state.set_playing(true);
                    player.notify_playback_status()?;
                    Ok(())
                })
                .unwrap();
            }
            _ => {}
        }
    }
}

fn register_mpris(
    connection: &zbus::Connection,
    player: SpotMprisPlayer,
) -> Result<zbus::ObjectServer, zbus::Error> {

    let mut object_server = zbus::ObjectServer::new(&connection);
    object_server.at(&"/org/mpris/MediaPlayer2".try_into()?, SpotMpris)?;
    object_server.at(
        &"/org/mpris/MediaPlayer2".try_into()?,
        player,
    )?;
    Ok(object_server)
}

pub fn start_dbus_server(sender: Sender<AppAction>) -> Result<ConnectionWrapper, zbus::Error> {
    let state = SharedMprisState::new();

    let connection = zbus::Connection::new_session()?;
    fdo::DBusProxy::new(&connection)?.request_name(
        "org.mpris.MediaPlayer2.Spot",
        fdo::RequestNameFlags::AllowReplacement.into(),
    )?;

    let state_clone = state.clone();
    let sender_clone = sender.clone();
    let conn_clone = connection.clone();

    thread::spawn(move || {
        let player = SpotMprisPlayer::new(state_clone, sender_clone);
        let mut object_server = register_mpris(&conn_clone, player).unwrap();
        loop {
            if let Err(err) = object_server.try_handle_next() {
                eprintln!("{}", err);
            }
        }
    });
    
    let player = SpotMprisPlayer::new(state, sender);
    ConnectionWrapper::new(connection, player)
}
