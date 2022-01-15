use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
use std::rc::Rc;
use std::thread;
use zbus::Connection;

use crate::app::{AppAction, AppModel};

mod mpris;
pub use mpris::*;

mod types;

mod listener;
use listener::*;

#[tokio::main]
async fn dbus_server(
    mpris: SpotMpris,
    player: SpotMprisPlayer,
    receiver: UnboundedReceiver<MprisStateUpdate>,
) -> zbus::Result<()> {
    let connection = Connection::session().await?;
    connection
        .object_server()
        .at("/org/mpris/MediaPlayer2", mpris)
        .await?;
    connection
        .object_server()
        .at("/org/mpris/MediaPlayer2", player)
        .await?;
    connection
        .request_name("org.mpris.MediaPlayer2.Spot")
        .await?;

    receiver
        .for_each(|update| async {
            if let Ok(player_ref) = connection
                .object_server()
                .interface::<_, SpotMprisPlayer>("/org/mpris/MediaPlayer2")
                .await
            {
                let mut player = player_ref.get_mut().await;
                let ctxt = player_ref.signal_context();
                let res: zbus::Result<()> = match update {
                    MprisStateUpdate::SetVolume(volume) => {
                        player.state_mut().set_volume(volume);
                        player.volume_changed(ctxt).await
                    }
                    MprisStateUpdate::SetCurrentTrack {
                        has_prev,
                        has_next,
                        current,
                    } => {
                        player.state_mut().set_has_prev(has_prev);
                        player.state_mut().set_has_next(has_next);
                        player.state_mut().set_current_track(current);
                        player.notify_current_track_changed(ctxt).await
                    }
                    MprisStateUpdate::SetPositionMs(position) => {
                        player.state_mut().set_position(position);
                        Ok(())
                    }
                    MprisStateUpdate::SetLoopStatus {
                        has_prev,
                        has_next,
                        loop_status,
                    } => {
                        player.state_mut().set_has_prev(has_prev);
                        player.state_mut().set_has_next(has_next);
                        player.state_mut().set_loop_status(loop_status);
                        player.notify_loop_status(ctxt).await
                    }
                    MprisStateUpdate::SetShuffled(shuffled) => {
                        player.state_mut().set_shuffled(shuffled);
                        player.shuffle_changed(ctxt).await
                    }
                    MprisStateUpdate::SetPlaying(status) => {
                        player.state_mut().set_playing(status);
                        player.playback_status_changed(ctxt).await
                    }
                };
                res.expect("Signal emission failed");
            }
        })
        .await;

    Ok(())
}

pub fn start_dbus_server(
    app_model: Rc<AppModel>,
    sender: UnboundedSender<AppAction>,
) -> AppPlaybackStateListener {
    let mpris = SpotMpris::new(sender.clone());
    let player = SpotMprisPlayer::new(sender);

    let (sender, receiver) = unbounded();

    thread::spawn(move || dbus_server(mpris, player, receiver));

    AppPlaybackStateListener::new(app_model, sender)
}
