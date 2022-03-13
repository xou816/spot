use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use librespot::core::spotify_id::SpotifyId;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;
use tokio::task;

use crate::app::credentials::Credentials;
use crate::app::state::{LoginAction, PlaybackAction, SetLoginSuccessAction};
use crate::app::AppAction;

mod player;
pub use player::*;

#[derive(Debug, Clone)]
pub enum Command {
    PasswordLogin { username: String, password: String },
    TokenLogin { username: String, token: String },
    Logout,
    PlayerLoad { track: SpotifyId, resume: bool },
    PlayerResume,
    PlayerPause,
    PlayerStop,
    PlayerSeek(u32),
    PlayerSetVolume(f64),
    PlayerPreload(SpotifyId),
    RefreshToken,
    ReloadSettings,
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

    fn password_login_successful(&self, credentials: Credentials) {
        self.sender
            .borrow_mut()
            .unbounded_send(
                LoginAction::SetLoginSuccess(SetLoginSuccessAction::Password(credentials)).into(),
            )
            .unwrap();
    }

    fn token_login_successful(&self, username: String, token: String) {
        self.sender
            .borrow_mut()
            .unbounded_send(
                LoginAction::SetLoginSuccess(SetLoginSuccessAction::Token { username, token })
                    .into(),
            )
            .unwrap();
    }

    fn refresh_successful(&self, token: String, token_expiry_time: SystemTime) {
        self.sender
            .borrow_mut()
            .unbounded_send(
                LoginAction::SetRefreshedToken {
                    token,
                    token_expiry_time,
                }
                .into(),
            )
            .unwrap();
    }

    fn report_error(&self, error: SpotifyError) {
        self.sender
            .borrow_mut()
            .unbounded_send(match error {
                SpotifyError::LoginFailed => LoginAction::SetLoginFailure.into(),
                _ => AppAction::ShowNotification(format!("{error}")),
            })
            .unwrap();
    }

    fn notify_playback_state(&self, position: u32) {
        self.sender
            .borrow_mut()
            .unbounded_send(PlaybackAction::SyncSeek(position).into())
            .unwrap();
    }

    fn preload_next_track(&self) {
        self.sender
            .borrow_mut()
            .unbounded_send(PlaybackAction::Preload.into())
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
