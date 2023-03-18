use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

use futures::channel::mpsc::UnboundedSender;
use gettextrs::gettext;

use crate::api::{SpotifyApiClient, SpotifyApiError, SpotifyResult};
use crate::app::models::{ConnectPlayerState, RepeatMode, SongDescription};
use crate::app::state::{Device, PlaybackAction};
use crate::app::{AppAction, SongsSource};

#[derive(Debug)]
pub enum ConnectCommand {
    SetDevice(String),
    PlayerLoadInContext {
        source: SongsSource,
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
    device_id: RwLock<Option<String>>,
    last_queue: RwLock<u64>,
    last_state: RwLock<ConnectPlayerState>,
}

impl ConnectPlayer {
    pub fn new(
        api: Arc<dyn SpotifyApiClient + Send + Sync>,
        action_sender: UnboundedSender<AppAction>,
    ) -> Self {
        Self {
            api: api.clone(),
            action_sender,
            device_id: Default::default(),
            last_queue: Default::default(),
            last_state: Default::default(),
        }
    }

    fn send_actions(&self, actions: impl IntoIterator<Item = AppAction>) {
        for action in actions.into_iter() {
            self.action_sender.unbounded_send(action).unwrap();
        }
    }

    fn device_lost(&self) {
        let _ = self.device_id.write().unwrap().take();
        self.send_actions([
            AppAction::ShowNotification(gettext("Connection to device lost!")),
            PlaybackAction::SwitchDevice(Device::Local).into(),
            PlaybackAction::SetAvailableDevices(vec![]).into(),
        ]);
    }

    async fn get_queue_if_changed(&self) -> Option<Vec<SongDescription>> {
        let last_queue = *self.last_queue.read().ok().as_deref().unwrap_or(&0u64);
        let songs = self.api.get_player_queue().await.ok();
        songs.filter(|songs| {
            let hash = {
                let mut hasher = DefaultHasher::new();
                songs.hash(&mut hasher);
                hasher.finish()
            };
            if let Some(last_queue) = self.last_queue.try_write().ok().as_deref_mut() {
                *last_queue = hash;
            }
            hash != last_queue
        })
    }

    async fn apply_remote_state(&self, state: &ConnectPlayerState) {
        if let Some(songs) = self.get_queue_if_changed().await {
            self.send_actions([PlaybackAction::LoadSongs(songs).into()]);
        }

        let play_pause = if state.is_playing {
            PlaybackAction::Load(state.current_song_id.clone().unwrap())
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

    pub fn has_device(&self) -> bool {
        self.device_id
            .read()
            .map(|it| it.is_some())
            .unwrap_or(false)
    }

    pub async fn sync_state(&self) {
        debug!("polling connect device...");
        let player_state = self.api.player_state().await;
        let Ok(state) = player_state else {
                self.device_lost();
                return;
        };
        self.apply_remote_state(&state).await;
        if let Ok(mut last_state) = self.last_state.write() {
            *last_state = state;
        }
    }

    async fn handle_player_load_in_context(
        &self,
        device_id: String,
        current_state: &ConnectPlayerState,
        command: ConnectCommand,
    ) -> SpotifyResult<()> {
        let ConnectCommand::PlayerLoadInContext {
            source,
            offset,
            song,
        } = command else {
            panic!("Illegal call");
        };
        let is_diff_song = current_state
            .current_song_id
            .as_ref()
            .map(|it| it != &song)
            .unwrap_or(true);
        let is_paused = !current_state.is_playing;
        if is_diff_song {
            let context = source.spotify_uri().unwrap();
            self.api
                .player_play_in_context(device_id, context, offset)
                .await
        } else if is_paused {
            self.api.player_resume(device_id).await
        } else {
            Ok(())
        }
    }

    async fn handle_player_load(
        &self,
        device_id: String,
        current_state: &ConnectPlayerState,
        command: ConnectCommand,
    ) -> SpotifyResult<()> {
        let ConnectCommand::PlayerLoad { songs, offset } = command else {
            panic!("Illegal call");
        };
        let is_diff_song = current_state
            .current_song_id
            .as_ref()
            .map(|it| it != &songs[offset])
            .unwrap_or(true);
        let is_paused = !current_state.is_playing;
        if is_diff_song {
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
        } else if is_paused {
            self.api.player_resume(device_id).await
        } else {
            Ok(())
        }
    }

    async fn handle_other_command(
        &self,
        device_id: String,
        command: ConnectCommand,
    ) -> SpotifyResult<()> {
        let state = self.last_state.read();
        let Ok(state) = state.as_deref() else {
            return Ok(());
        };
        match command {
            ConnectCommand::PlayerLoadInContext { .. } => {
                self.handle_player_load_in_context(device_id, state, command)
                    .await
            }
            ConnectCommand::PlayerLoad { .. } => {
                self.handle_player_load(device_id, state, command).await
            }
            ConnectCommand::PlayerResume if !state.is_playing => {
                self.api.player_resume(device_id).await
            }
            ConnectCommand::PlayerPause if state.is_playing => {
                self.api.player_pause(device_id).await
            }
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
                self.device_id.write().ok()?.replace(new_device_id);
                self.sync_state().await;
                false
            }
            ConnectCommand::PlayerStop => {
                let device_id = self.device_id.write().ok()?.take();
                if let Some(old_id) = device_id {
                    let _ = self.api.player_pause(old_id).await;
                }
                false
            }
            _ => {
                let device_id = self.device_id.read().ok()?.clone();
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
