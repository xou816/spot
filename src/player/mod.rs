use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use librespot::core::spotify_id::SpotifyId;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::task;
use url::Url;

use crate::app::state::{LoginAction, PlaybackAction};
use crate::app::AppAction;
#[allow(clippy::module_inception)]
mod player;
pub use player::*;

mod oauth2;

mod token_store;
pub use token_store::*;

#[derive(Debug, Clone)]
pub enum Command {
    Restore,
    InitLogin,
    CompleteLogin,
    RefreshToken,
    Logout,
    PlayerLoad { track: SpotifyId, resume: bool },
    PlayerResume,
    PlayerPause,
    PlayerStop,
    PlayerSeek(u32),
    PlayerSetVolume(f64),
    PlayerPreload(SpotifyId),
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

    fn token_login_successful(&self, username: String) {
        self.sender
            .borrow_mut()
            .unbounded_send(LoginAction::SetLoginSuccess(username).into())
            .unwrap();
    }

    fn refresh_successful(&self) {
        self.sender
            .borrow_mut()
            .unbounded_send(LoginAction::TokenRefreshed.into())
            .unwrap();
    }

    fn report_error(&self, error: SpotifyError) {
        self.sender
            .borrow_mut()
            .unbounded_send(match error {
                SpotifyError::LoginFailed => LoginAction::SetLoginFailure.into(),
                SpotifyError::LoggedOut => LoginAction::Logout.into(),
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

    fn login_challenge_started(&self, url: Url) {
        self.sender
            .borrow_mut()
            .unbounded_send(LoginAction::OpenLoginUrl(url).into())
            .unwrap();
    }
}

#[tokio::main]
async fn player_main(
    player_settings: SpotifyPlayerSettings,
    appaction_sender: UnboundedSender<AppAction>,
    token_store: Arc<TokenStore>,
    sender: UnboundedSender<Command>,
    receiver: UnboundedReceiver<Command>,
) {
    task::LocalSet::new()
        .run_until(async move {
            task::spawn_local(async move {
                let delegate = Rc::new(AppPlayerDelegate::new(appaction_sender.clone()));
                let player = SpotifyPlayer::new(player_settings, delegate, token_store, sender);
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
    token_store: Arc<TokenStore>,
) -> UnboundedSender<Command> {
    let (sender, receiver) = unbounded::<Command>();
    let sender_clone = sender.clone();
    std::thread::spawn(move || {
        player_main(
            player_settings,
            appaction_sender,
            token_store,
            sender_clone,
            receiver,
        )
    });
    sender
}
