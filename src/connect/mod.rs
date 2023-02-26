use std::sync::Arc;
use std::time::Duration;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
use tokio::{task, time};

use crate::api::SpotifyApiClient;
use crate::app::AppAction;

mod player;
pub use player::ConnectCommand;

#[tokio::main]
async fn connect_server(
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
    action_sender: UnboundedSender<AppAction>,
    receiver: UnboundedReceiver<ConnectCommand>,
) {
    let player = Arc::new(player::ConnectPlayer::new(api, action_sender));

    let player_clone = Arc::clone(&player);
    task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            player_clone.sync_state().await;
        }
    });

    receiver
        .for_each(|command| async { player.handle_command(command).await.unwrap() })
        .await;
}

pub fn start_connect_server(
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
    action_sender: UnboundedSender<AppAction>,
) -> UnboundedSender<ConnectCommand> {
    let (sender, receiver) = unbounded();

    std::thread::spawn(move || connect_server(api, action_sender, receiver));

    sender
}
