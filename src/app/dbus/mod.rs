use futures::channel::mpsc::Sender;
use std::convert::TryInto;
use std::error::Error;
use std::thread;
use zbus::fdo;

mod mpris;
use mpris::*;

use crate::app::AppAction;

fn register_dbus_service(
    appaction_sender: Sender<AppAction>,
) -> Result<zbus::ObjectServer<'static>, Box<dyn Error>> {
    let connection = zbus::Connection::new_session()?;
    fdo::DBusProxy::new(&connection)?.request_name(
        "org.mpris.MediaPlayer2.Spot",
        fdo::RequestNameFlags::ReplaceExisting.into(),
    )?;

    let mut object_server = zbus::ObjectServer::new(&connection);
    object_server.at(&"/org/mpris/MediaPlayer2".try_into()?, SpotMpris(appaction_sender.clone()))?;
    object_server.at(&"/org/mpris/MediaPlayer2".try_into()?, SpotMprisPlayer(appaction_sender))?;
    Ok(object_server)
}

pub fn start_dbus_service(appaction_sender: Sender<AppAction>) {
    thread::spawn(move || {
        let mut object_server = register_dbus_service(appaction_sender).unwrap();
        loop {
            if let Err(err) = object_server.try_handle_next() {
                eprintln!("{}", err);
            }
        }
    });
}
