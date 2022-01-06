use futures::channel::mpsc::UnboundedReceiver;
use futures::stream::StreamExt;

use librespot::core::authentication::Credentials;
use librespot::core::cache::Cache;
use librespot::core::config::SessionConfig;
use librespot::core::keymaster;
use librespot::core::session::{Session, SessionError};

use librespot::playback::mixer::softmixer::SoftMixer;
use librespot::playback::mixer::{Mixer, MixerConfig};
use librespot::protocol::authentication::AuthenticationType;

use librespot::playback::audio_backend;
use librespot::playback::config::{AudioFormat, Bitrate, PlayerConfig, VolumeCtrl};
use librespot::playback::player::{Player, PlayerEvent, PlayerEventChannel};

use std::cell::RefCell;
use std::env;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use super::Command;
use crate::app::credentials;
use crate::app::state::Device;
use crate::settings::SpotSettings;

#[derive(Debug)]
pub enum SpotifyError {
    LoginFailed,
    TokenFailed,
    PlayerNotReady,
    TechnicalError,
}

impl Error for SpotifyError {}

impl fmt::Display for SpotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoginFailed => write!(f, "Login failed!"),
            Self::TokenFailed => write!(f, "Token retrieval failed!"),
            Self::PlayerNotReady => write!(f, "Player is not responding."),
            Self::TechnicalError => {
                write!(f, "A technical error occured. Check your connectivity.")
            }
        }
    }
}

pub trait SpotifyPlayerDelegate {
    fn end_of_track_reached(&self);
    fn password_login_successful(&self, credentials: credentials::Credentials);
    fn token_login_successful(&self, username: String, token: String);
    fn refresh_successful(&self, token: String, token_expiry_time: SystemTime);
    fn report_error(&self, error: SpotifyError);
    fn notify_playback_state(&self, position: u32);
    fn preload_next_track(&self);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioBackend {
    GStreamer(String),
    PulseAudio,
    Alsa(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpotifyPlayerSettings {
    pub bitrate: Bitrate,
    pub backend: AudioBackend,
    pub gapless: bool,
    pub ap_port: Option<u16>,
}

impl Default for SpotifyPlayerSettings {
    fn default() -> Self {
        Self {
            bitrate: Bitrate::Bitrate160,
            gapless: true,
            backend: AudioBackend::PulseAudio,
            ap_port: None,
        }
    }
}

pub struct SpotifyPlayer {
    api: Arc<dyn SpotifyApiClient + Send + Sync>,
    settings: SpotifyPlayerSettings,
    player: Option<Player>,
    mixer: Option<Box<dyn Mixer>>,
    session: Option<Session>,
    delegate: Rc<dyn SpotifyPlayerDelegate>,
    device: Device,
}

impl SpotifyPlayer {
    pub fn new(
        api: Arc<dyn SpotifyApiClient + Send + Sync>,
        settings: SpotifyPlayerSettings,
        delegate: Rc<dyn SpotifyPlayerDelegate>,
    ) -> Self {
        Self {
            api,
            settings,
            mixer: None,
            player: None,
            session: None,
            delegate,
            device: Device::Connect,
        }
    }

    async fn handle(&mut self, action: Command) -> Result<(), SpotifyError> {
        match (self.device, action) {
            (Device::Local, Command::PlayerSetVolume(volume)) => {
                if let Some(mixer) = self.mixer.as_mut() {
                    mixer.set_volume((VolumeCtrl::MAX_VOLUME as f64 * volume) as u16);
                }
                Ok(())
            }
            (Device::Local, Command::PlayerResume)) => {
                self.player
                    .as_ref()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .play();
                Ok(())
            }
            (Device::Connect, Command::PlayerResume) => {
                self.api.player_play(None).await.unwrap();
                Ok(())
            }
            (Device::Local, Command::PlayerPause)) => {
                self.player
                    .as_ref()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .pause();
                Ok(())
            }
            (Device::Connect, Command::PlayerPause) => {
                self.api.player_pause().await.unwrap();
                Ok(())
            }
            (Device::Local, Command::PlayerStop)) => {
                self.player
                    .as_ref()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .stop();
                Ok(())
            }
            (Device::Local, Command::PlayerSeek(position)) => {
                self.player
                    .as_ref()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .seek(position);
                Ok(())
            }
            (Device::Connect, Command::PlayerSeek(position)) => {
                self.api.player_seek(position as usize).await.unwrap();
                Ok(())
            }
            (Device::Local, Command::PlayerLoad(track)) => {
                self.player
                    .as_mut()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .load(track, true, 0);
                Ok(())
            }
            (Device::Connect, Command::PlayerLoad(track)) => {
                let uri = track.to_uri();
                self.api.player_play(Some(uri)).await.unwrap();
                Ok(())
            }
            (Device::Local, Command::PlayerPreload(track)) => {
                self.player
                    .as_mut()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .preload(track);
                Ok(())
            }
            (Device::Local, Command::SwitchDevice(Device::Local)) => Ok(()),
            (Device::Local, Command::SwitchDevice(Device::Connect)) => {
                let player = player.as_mut().ok_or(SpotifyError::PlayerNotReady)?;
                player.pause();
                *self.device.borrow_mut() = Device::Connect;
                self.api.player_play(None).await.unwrap();
                Ok(())
            }
            (Device::Connect, Command::SwitchDevice(Device::Local)) => {
                let player = player.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                self.api.player_pause().await.unwrap();
                *self.device.borrow_mut() = Device::Local;
                player.play();
                Ok(())
            }
            (Device::Connect, Command::SwitchDevice(Device::Connect)) => Ok(()),
            (_, Command::RefreshToken) => {
                let session = self.session.as_ref().ok_or(SpotifyError::PlayerNotReady)?;
                let (token, token_expiry_time) = get_access_token_and_expiry_time(session).await?;
                self.delegate.refresh_successful(token, token_expiry_time);
                Ok(())
            }
            (_, Command::Logout) => {
                self.session
                    .take()
                    .ok_or(SpotifyError::PlayerNotReady)?
                    .shutdown();
                let _ = self.player.take();
                Ok(())
            }
            Command::PasswordLogin { username, password } => {
                let credentials = Credentials::with_password(username, password.clone());
                let new_session = create_session(&credentials, self.settings.ap_port).await?;
                let (token, token_expiry_time) =
                    get_access_token_and_expiry_time(&new_session).await?;
                let credentials = credentials::Credentials {
                    username: new_session.username(),
                    password,
                    token,
                    token_expiry_time: Some(token_expiry_time),
                    country: new_session.country(),
                };
                self.delegate.password_login_successful(credentials);

                let (new_player, channel) = self.create_player(new_session.clone());
                tokio::task::spawn_local(player_setup_delegate(channel, Rc::clone(&self.delegate)));
                self.player.replace(new_player);
                self.session.replace(new_session);

                Ok(())
            }
            Command::TokenLogin { username, token } => {
                let credentials = Credentials {
                    username,
                    auth_type: AuthenticationType::AUTHENTICATION_SPOTIFY_TOKEN,
                    auth_data: token.clone().into_bytes(),
                };
                let new_session = create_session(&credentials, self.settings.ap_port).await?;
                self.delegate
                    .token_login_successful(new_session.username(), token);

                let (new_player, channel) = self.create_player(new_session.clone());
                tokio::task::spawn_local(player_setup_delegate(channel, Rc::clone(&self.delegate)));
                self.player.replace(new_player);
                self.session.replace(new_session);

                Ok(())
            }
            Command::ReloadSettings => {
                let settings = SpotSettings::new_from_gsettings().unwrap_or_default();
                self.settings = settings.player_settings;

                let session = self.session.take().ok_or(SpotifyError::PlayerNotReady)?;
                let (new_player, channel) = self.create_player(session);
                tokio::task::spawn_local(player_setup_delegate(channel, Rc::clone(&self.delegate)));
                self.player.replace(new_player);

                Ok(())
            }
        }
    }

    fn create_player(&mut self, session: Session) -> (Player, PlayerEventChannel) {
        let backend = self.settings.backend.clone();

        let player_config = PlayerConfig {
            gapless: self.settings.gapless,
            bitrate: self.settings.bitrate,
            ..Default::default()
        };
        info!("bitrate: {:?}", &player_config.bitrate);

        let soft_volume = self
            .mixer
            .get_or_insert_with(|| {
                let mix = Box::new(SoftMixer::open(MixerConfig {
                    // This value feels reasonable to me. Feel free to change it
                    volume_ctrl: VolumeCtrl::Log(VolumeCtrl::DEFAULT_DB_RANGE / 2.0),
                    ..Default::default()
                }));
                // TODO: Should read volume from somewhere instead of hard coding.
                // Sets volume to 100%
                mix.set_volume(VolumeCtrl::MAX_VOLUME);
                mix
            })
            .get_soft_volume();
        Player::new(player_config, session, soft_volume, move || match backend {
            AudioBackend::GStreamer(pipeline) => {
                let backend = audio_backend::find(Some("gstreamer".to_string())).unwrap();
                backend(Some(pipeline), AudioFormat::default())
            }
            AudioBackend::PulseAudio => {
                info!("using pulseaudio");
                env::set_var("PULSE_PROP_application.name", "Spot");
                let backend = audio_backend::find(Some("pulseaudio".to_string())).unwrap();
                backend(None, AudioFormat::default())
            }
            AudioBackend::Alsa(device) => {
                info!("using alsa ({})", &device);
                let backend = audio_backend::find(Some("alsa".to_string())).unwrap();
                backend(Some(device), AudioFormat::default())
            }
        })
    }

    pub async fn start(self, receiver: UnboundedReceiver<Command>) -> Result<(), ()> {
        let _self = RefCell::new(self);
        receiver
            .for_each(|action| async {
                let mut _self = _self.borrow_mut();
                match _self.handle(action).await {
                    Ok(_) => {}
                    Err(err) => _self.delegate.report_error(err),
                }
            })
            .await;
        Ok(())
    }
}

const CLIENT_ID: &str = "782ae96ea60f4cdf986a766049607005";

const SCOPES: &str = "user-read-private,\
playlist-read-private,\
playlist-read-collaborative,\
user-library-read,\
user-library-modify,\
user-top-read,\
user-read-recently-played,\
user-read-playback-state,\
playlist-modify-public,\
playlist-modify-private,\
user-modify-playback-state,\
streaming,\
playlist-modify-public";

const KNOWN_AP_PORTS: [Option<u16>; 4] = [None, Some(80), Some(443), Some(4070)];

async fn get_access_token_and_expiry_time(
    session: &Session,
) -> Result<(String, SystemTime), SpotifyError> {
    let token = keymaster::get_token(session, CLIENT_ID, SCOPES)
        .await
        .map_err(|_e| SpotifyError::TokenFailed)?;
    let expiry_time = SystemTime::now() + Duration::from_secs(token.expires_in.into());
    Ok((token.access_token, expiry_time))
}

async fn create_session_with_port(
    credentials: &Credentials,
    ap_port: Option<u16>,
) -> Result<Session, SpotifyError> {
    let session_config = SessionConfig {
        ap_port,
        ..Default::default()
    };
    let root = glib::user_cache_dir().join("spot").join("librespot");
    let cache = Cache::new(
        Some(root.join("credentials")),
        Some(root.join("volume")),
        Some(root.join("audio")),
        None,
    )
    .map_err(|e| dbg!(e))
    .ok();
    match Session::connect(session_config, credentials.clone(), cache, true).await {
        Ok(r) => Ok(r.0),
        Err(SessionError::IoError(_)) => Err(SpotifyError::TechnicalError),
        Err(SessionError::AuthenticationError(err)) => {
            warn!("Login failure: {}", err);
            Err(SpotifyError::LoginFailed)
        }
    }
}

async fn create_session(
    credentials: &Credentials,
    ap_port: Option<u16>,
) -> Result<Session, SpotifyError> {
    match ap_port {
        Some(_) => create_session_with_port(credentials, ap_port).await,
        None => {
            let mut ports_to_try = KNOWN_AP_PORTS.iter();
            loop {
                if let Some(next_port) = ports_to_try.next() {
                    let res = create_session_with_port(credentials, *next_port).await;
                    match res {
                        Err(SpotifyError::TechnicalError) => continue,
                        _ => break res,
                    }
                } else {
                    break Err(SpotifyError::TechnicalError);
                }
            }
        }
    }
}

async fn player_setup_delegate(
    mut channel: PlayerEventChannel,
    delegate: Rc<dyn SpotifyPlayerDelegate>,
) {
    while let Some(event) = channel.recv().await {
        match event {
            PlayerEvent::EndOfTrack { .. } | PlayerEvent::Stopped { .. } => {
                delegate.end_of_track_reached();
            }
            PlayerEvent::Playing { position_ms, .. } => {
                delegate.notify_playback_state(position_ms);
            }
            PlayerEvent::TimeToPreloadNextTrack { .. } => {
                debug!("Requestiong next track to be preloaded...");
                delegate.preload_next_track();
            }
            _ => {}
        }
    }
}
