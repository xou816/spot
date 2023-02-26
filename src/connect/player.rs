use std::sync::{Arc, Mutex};

use futures::channel::mpsc::UnboundedSender;
use gettextrs::gettext;

use crate::api::{SpotifyApiClient, SpotifyApiError, SpotifyResult};
use crate::app::models::RepeatMode;
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
    PlayerLoad {
        songs: Vec<String>,
        offset: usize,
    },
    PlayerResume,
    PlayerPause,
    PlayerStop,
    PlayerSeek(usize),
    PlayerRepeat(RepeatMode),
    PlayerShuffle(bool),
    PlayerSetVolume(u8),
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

    async fn should_play(&self, song: &str) -> bool {
        let current_state = self.api.player_state().await.ok();
        current_state
            .and_then(|s| s.current_song_id)
            .map(|id| id != song)
            .unwrap_or(true)
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
                let should_play = self.should_play(song.as_str()).await;
                if should_play {
                    self.api
                        .player_play_in_context(device_id, context, offset)
                        .await
                } else {
                    self.api.player_resume(device_id).await
                }
            }
            ConnectCommand::PlayerLoad { songs, offset } => {
                let should_play = self.should_play(songs[offset].as_str()).await;
                if should_play {
                    self.api
                        .player_play_no_context(
                            device_id,
                            songs
                                .into_iter()
                                .map(|s| format!("spotify:track:{}", s))
                                .collect(),
                            offset,
                        )
                        .await
                } else {
                    self.api.player_resume(device_id).await
                }
            }
            ConnectCommand::PlayerResume => self.api.player_resume(device_id).await,
            ConnectCommand::PlayerPause => self.api.player_pause(device_id).await,
            ConnectCommand::PlayerSeek(offset) => self.api.player_seek(device_id, offset).await,
            ConnectCommand::PlayerRepeat(mode) => self.api.player_repeat(device_id, mode).await,
            ConnectCommand::PlayerShuffle(shuffle) => {
                self.api.player_shuffle(device_id, shuffle).await
            }
            ConnectCommand::PlayerSetVolume(volume) => {
                self.api.player_volume(device_id, volume).await
            }
            _ => Ok(()),
        }
    }

    pub async fn handle_command(&self, command: ConnectCommand) -> Option<()> {
        let device_lost = match command {
            ConnectCommand::SetDevice(new_device_id) => {
                self.device_id.lock().ok()?.replace(new_device_id);
                false
            }
            ConnectCommand::PlayerStop => {
                let device_id = self.device_id.lock().ok()?.take();
                if let Some(old_id) = device_id {
                    let _ = self.api.player_pause(old_id).await;
                }
                false
            }
            _ => {
                let device_id = self.device_id.lock().ok()?.clone();
                if let Some(device_id) = device_id {
                    let result = self.handle_other_command(device_id, command).await;
                    matches!(result, Err(SpotifyApiError::BadStatus(404, _)))
                } else {
                    true
                }
            }
        };

        if device_lost {
            self.device_lost();
        }

        Some(())
    }
}
