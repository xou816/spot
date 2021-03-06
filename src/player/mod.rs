use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use librespot::core::spotify_id::SpotifyId;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::task;

use crate::app::state::{LoginAction, PlaybackAction};
use crate::app::{credentials, AppAction};

mod player;
pub use player::*;

#[derive(Debug, Clone)]
pub enum Command {
    Login(String, String),
    Logout,
    PlayerLoad(SpotifyId),
    PlayerResume,
    PlayerPause,
    PlayerStop,
    PlayerSeek(u32),
    RefreshToken,
}

struct AppPlayerDelegate {
    sender: RefCell<UnboundedSender<AppAction>>,
}

impl AppPlayerDelegate {
    fn new(sender: UnboundedSender<AppAction>) -> Self {
        let sender = RefCell::new(sender);
        Self { sender }
    }
}

impl SpotifyPlayerDelegate for AppPlayerDelegate {
    fn end_of_track_reached(&self) {
        self.sender
            .borrow_mut()
            .unbounded_send(PlaybackAction::Next.into())
            .unwrap();
    }

    fn login_successful(&self, credentials: credentials::Credentials) {
        self.sender
            .borrow_mut()
            .unbounded_send(LoginAction::SetLoginSuccess(credentials).into())
            .unwrap();
    }

    fn refresh_successful(&self, token: String) {
        self.sender
            .borrow_mut()
            .unbounded_send(LoginAction::SetRefreshedToken(token).into())
            .unwrap();
    }

    fn report_error(&self, error: SpotifyError) {
        self.sender
            .borrow_mut()
            .unbounded_send(match error {
                SpotifyError::LoginFailed => LoginAction::SetLoginFailure.into(),
                _ => AppAction::ShowNotification(format!("{}", error)),
            })
            .unwrap();
    }

    fn notify_playback_state(&self, position: u32) {
        self.sender
            .borrow_mut()
            .unbounded_send(PlaybackAction::SyncSeek(position).into())
            .unwrap();
    }
}

#[tokio::main]
async fn player_main(
    player_settings: SpotifyPlayerSettings,
    appaction_sender: UnboundedSender<AppAction>,
    receiver: UnboundedReceiver<Command>,
) {
    task::LocalSet::new()
        .run_until(async move {
            task::spawn_local(async move {
                let delegate = Rc::new(AppPlayerDelegate::new(appaction_sender.clone()));
                let player = SpotifyPlayer::new(player_settings, delegate);
                player.start(receiver).await.unwrap();
            })
            .await
            .unwrap();
        })
        .await;
}

pub fn start_player_service(
    player_settings: SpotifyPlayerSettings,
    appaction_sender: UnboundedSender<AppAction>,
) -> UnboundedSender<Command> {
    let (sender, receiver) = unbounded::<Command>();
    std::thread::spawn(move || player_main(player_settings, appaction_sender, receiver));
    sender
}
