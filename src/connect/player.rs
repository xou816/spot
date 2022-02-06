use std::sync::{Arc, Mutex};

use futures::channel::mpsc::UnboundedSender;
use gettextrs::gettext;

use crate::api::{SpotifyApiClient, SpotifyApiError, SpotifyResult};
use crate::app::state::{Device, PlaybackAction};
use crate::app::AppAction;

#[derive(Debug)]
pub enum ConnectCommand {
    SetDevice(String),
    PlayerLoadInContext {
        context: String,
        offset: usize,
        song: String,
    },
    PlayerLoad(String),
    PlayerResume,
    PlayerPause,
    PlayerStop,
    PlayerSeek(usize),
}

pub struct ConnectPlayer {
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
    action_sender: UnboundedSender<AppAction>,
    device_id: Mutex<Option<String>>,
}

impl ConnectPlayer {
    pub fn new(
        api: Arc<dyn SpotifyApiClient + Send + Sync>,
        action_sender: UnboundedSender<AppAction>,
    ) -> Self {
        Self {
            api,
            action_sender,
            device_id: Default::default(),
        }
    }

    fn send_actions(&self, actions: impl IntoIterator<Item = AppAction>) {
        for action in actions.into_iter() {
            self.action_sender.unbounded_send(action).unwrap();
        }
    }

    fn device_lost(&self) {
        let _ = self.device_id.lock().unwrap().take();
        self.send_actions([
            AppAction::ShowNotification(gettext("Connection to device lost!")),
            PlaybackAction::SwitchDevice(Device::Local).into(),
            PlaybackAction::SetAvailableDevices(vec![]).into(),
        ]);
    }

    async fn sync_state_unguarded(&self) {
        let state = self.api.player_state().await;
        match state {
            Ok(state) => {
                let play_pause = if state.is_playing {
                    PlaybackAction::Load(state.current_song_id.unwrap())
                } else {
                    PlaybackAction::Pause
                };
                self.send_actions([
                    play_pause.into(),
                    PlaybackAction::SetRepeatMode(state.repeat).into(),
                    PlaybackAction::SetShuffled(state.shuffle).into(),
                    PlaybackAction::SyncSeek(state.progress_ms).into(),
                ]);
            }
            Err(SpotifyApiError::NoContent) => {
                self.device_lost();
            }
            _ => {}
        }
    }

    pub async fn sync_state(&self) {
        let device_id = self.device_id.try_lock().ok().and_then(|d| (*d).clone());
        if device_id.is_some() {
            debug!("polling connect device...");
            self.sync_state_unguarded().await;
        }
    }

    async fn handle_other_command(
        &self,
        device_id: String,
        command: ConnectCommand,
    ) -> SpotifyResult<()> {
        match command {
            ConnectCommand::PlayerLoadInContext {
                context,
                offset,
                song,
            } => {
                let current_state = self.api.player_state().await.ok();
                let should_play = current_state
                    .and_then(|s| s.current_song_id)
                    .map(|id| id != song)
                    .unwrap_or(true);
                if should_play {
                    self.api
                        .player_play_in_context(device_id, context, offset)
                        .await
                } else {
                    Ok(())
                }
            }
            ConnectCommand::PlayerResume => self.api.player_resume(device_id).await,
            ConnectCommand::PlayerPause => self.api.player_pause(device_id).await,
            ConnectCommand::PlayerSeek(offset) => self.api.player_seek(device_id, offset).await,
            _ => Ok(()),
        }
    }

    pub async fn handle_command(&self, command: ConnectCommand) {
        let device_lost = {
            let mut device_id = self.device_id.lock().unwrap();
            if let ConnectCommand::SetDevice(new_device_id) = command {
                *device_id = Some(new_device_id);
                false
            } else if let ConnectCommand::PlayerStop = command {
                if let Some(old_id) = device_id.take() {
                    let _ = self.api.player_pause(old_id).await;
                }
                false
            } else if let Some(device_id) = &*device_id {
                let result = self.handle_other_command(device_id.clone(), command).await;
                matches!(result, Err(SpotifyApiError::BadStatus(404, _)))
            } else {
                true
            }
        };

        if device_lost {
            self.device_lost();
        }
    }
}
