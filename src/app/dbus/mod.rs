use futures::channel::mpsc::UnboundedSender;
use std::convert::TryInto;
use std::rc::Rc;
use std::thread;
use zbus::fdo;

use crate::app::components::EventListener;
use crate::app::{models::SongDescription, AppAction, AppEvent, AppModel};

mod mpris;
pub use mpris::*;

mod types;
use types::*;

// This one wraps a connection and reads the app state
pub struct ConnectionWrapper {
    object_server: zbus::ObjectServer,
    app_model: Rc<AppModel>,
}

impl ConnectionWrapper {
    fn new(
        connection: zbus::Connection,
        player: SpotMprisPlayer,
        app_model: Rc<AppModel>,
    ) -> Result<Self, zbus::Error> {
        let object_server = register_mpris(&connection, player)?;
        Ok(Self {
            object_server,
            app_model,
        })
    }

    fn with_player<F: Fn(&SpotMprisPlayer) -> zbus::Result<()>>(&self, f: F) -> zbus::Result<()> {
        self.object_server.with(
            &"/org/mpris/MediaPlayer2".try_into()?,
            |iface: &SpotMprisPlayer| f(iface),
        )
    }

    fn make_track_meta(&self) -> Option<TrackMetadata> {
        self.app_model.get_state().current_song().map(
            |SongDescription {
                 id,
                 title,
                 artists,
                 duration,
                 ..
             }| TrackMetadata {
                id: format!("/dev/alextren/Spot/Track/{}", id),
                length: duration as u64,
                title,
                artist: artists.into_iter().map(|a| a.name).collect(),
            },
        )
    }

    fn has_prev_next(&self) -> (bool, bool) {
        let state = self.app_model.get_state();
        (state.prev_song().is_some(), state.next_song().is_some())
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
            AppEvent::TrackChanged(_) => {
                self.with_player(|player| {
                    let meta = self.make_track_meta();
                    let (has_prev, has_next) = self.has_prev_next();
                    player.state.set_current_track(meta);
                    player.state.set_has_prev(has_prev);
                    player.state.set_has_next(has_next);
                    player.notify_metadata_and_prev_next()?;
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
    object_server.at(&"/org/mpris/MediaPlayer2".try_into()?, player)?;
    Ok(object_server)
}

pub fn start_dbus_server(
    app_model: Rc<AppModel>,
    sender: UnboundedSender<AppAction>,
) -> Result<ConnectionWrapper, zbus::Error> {
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
    ConnectionWrapper::new(connection, player, app_model)
}
