use std::sync::{Arc, Mutex};

use futures::channel::mpsc::UnboundedSender;
use gettextrs::gettext;

use crate::api::{SpotifyApiClient, SpotifyApiError, SpotifyResult};
use crate::app::models::{Batch, ConnectPlayerState, RepeatMode};
use crate::app::state::{Device, PlaybackAction};
use crate::app::{AppAction, BatchLoader, BatchQuery, SongsSource};

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

#[derive(Default, Clone)]
struct State {
    device_id: Option<String>,
    source: Option<SongsSource>,
}

pub struct ConnectPlayer {
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
    batch_loader: BatchLoader,
    action_sender: UnboundedSender<AppAction>,
    state: Mutex<State>,
}

impl ConnectPlayer {
    pub fn new(
        api: Arc<dyn SpotifyApiClient + Send + Sync>,
        action_sender: UnboundedSender<AppAction>,
    ) -> Self {
        Self {
            api: api.clone(),
            batch_loader: BatchLoader::new(api),
            action_sender,
            state: Default::default(),
        }
    }

    fn send_actions(&self, actions: impl IntoIterator<Item = AppAction>) {
        for action in actions.into_iter() {
            self.action_sender.unbounded_send(action).unwrap();
        }
    }

    fn device_lost(&self) {
        let _ = self.state.lock().unwrap().device_id.take();
        self.send_actions([
            AppAction::ShowNotification(gettext("Connection to device lost!")),
            PlaybackAction::SwitchDevice(Device::Local).into(),
            PlaybackAction::SetAvailableDevices(vec![]).into(),
        ]);
    }

    async fn sync_state_send_actions(&self, partial_state: State, state: &ConnectPlayerState) {
        let load_songs = match (state.source.as_ref(), partial_state.source.as_ref()) {
            (Some(source), old_source) if Some(source) != old_source => {
                let query = BatchQuery {
                    source: source.clone(),
                    batch: Batch::first_of_size(50),
                };
                let action = self
                    .batch_loader
                    .query(query, |source, batch| {
                        PlaybackAction::LoadPagedSongs(source, batch).into()
                    })
                    .await;
                Some(action)
            }
            _ => None,
        };

        if let Some(load_songs) = load_songs {
            self.send_actions([load_songs]);
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

    pub async fn sync_state(&self) -> Option<()> {
        let partial_state = self.state.try_lock().ok()?.clone();
        let has_device = partial_state.device_id.is_some();
        if has_device {
            debug!("polling connect device...");
            let player_state = self.api.player_state().await;
            let Ok(state) = player_state else {
                self.device_lost();
                return Some(());
            };
            self.sync_state_send_actions(partial_state, &state).await;
            self.state.lock().ok()?.source = state.source;
        }
        Some(())
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
                source,
                offset,
                song,
            } => {
                let should_play = self.should_play(song.as_str()).await;
                if should_play {
                    let context = source.spotify_uri().unwrap();
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
                self.state.lock().ok()?.device_id.replace(new_device_id);
                false
            }
            ConnectCommand::PlayerStop => {
                let device_id = self.state.lock().ok()?.device_id.take();
                if let Some(old_id) = device_id {
                    let _ = self.api.player_pause(old_id).await;
                }
                false
            }
            _ => {
                let device_id = self.state.lock().ok()?.device_id.clone();
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
