use std::borrow::Cow;

use crate::app::state::sidebar_state::{SidebarAction, SidebarEvent};
use crate::app::state::{
    browser_state::{BrowserAction, BrowserEvent, BrowserState},
    login_state::{LoginAction, LoginEvent, LoginState},
    playback_state::{PlaybackAction, PlaybackEvent, PlaybackState},
    selection_state::{SelectionAction, SelectionContext, SelectionEvent, SelectionState},
    settings_state::{SettingsAction, SettingsEvent, SettingsState},
    sidebar_state::SidebarState,
    ScreenName, UpdatableState,
};

#[derive(Clone, Debug)]
pub enum AppAction {
    PlaybackAction(PlaybackAction),
    BrowserAction(BrowserAction),
    SelectionAction(SelectionAction),
    LoginAction(LoginAction),
    SettingsAction(SettingsAction),
    Start,
    Raise,
    ShowNotification(String),
    ShowPlaylistCreatedNotification(String, String, String),
    ViewNowPlaying,
    // cross-state actions
    QueueSelection,
    DequeueSelection,
    MoveUpSelection,
    MoveDownSelection,
    SaveSelection,
    UnsaveSelection,
    EnableSelection(SelectionContext),
    CancelSelection,
    SidebarAction(SidebarAction),
}

impl AppAction {
    #[allow(non_snake_case)]
    pub fn OpenURI(uri: String) -> Option<Self> {
        debug!("parsing {}", &uri);
        let mut parts = uri.split(':');
        if parts.next()? != "spotify" {
            return None;
        }

        // Might start with /// because of https://gitlab.gnome.org/GNOME/glib/-/issues/1886/
        let action = parts
            .next()?
            .strip_prefix("///")
            .filter(|p| !p.is_empty())?;
        let data = parts.next()?;

        match action {
            "album" => Some(Self::ViewAlbum(data.to_string())),
            "artist" => Some(Self::ViewArtist(data.to_string())),
            "playlist" => Some(Self::ViewPlaylist(data.to_string())),
            "user" => Some(Self::ViewUser(data.to_string())),
            _ => None,
        }
    }

    #[allow(non_snake_case)]
    pub fn ViewAlbum(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::AlbumDetails(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewArtist(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::Artist(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewPlaylist(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::PlaylistDetails(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewUser(id: String) -> Self {
        BrowserAction::NavigationPush(ScreenName::User(id)).into()
    }

    #[allow(non_snake_case)]
    pub fn ViewSearch() -> Self {
        BrowserAction::NavigationPush(ScreenName::Search).into()
    }
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    PlaybackEvent(PlaybackEvent),
    BrowserEvent(BrowserEvent),
    SelectionEvent(SelectionEvent),
    LoginEvent(LoginEvent),
    Started,
    Raised,
    NotificationShown(String),
    PlaylistCreatedNotificationShown(String, String, String),
    NowPlayingShown,
    SettingsEvent(SettingsEvent),
    SidebarEvent(SidebarEvent),
}

pub struct AppState {
    started: bool,
    pub playback: PlaybackState,
    pub browser: BrowserState,
    pub selection: SelectionState,
    pub logged_user: LoginState,
    pub settings: SettingsState,
    pub sidebar: SidebarState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            started: false,
            playback: Default::default(),
            browser: BrowserState::new(),
            selection: Default::default(),
            logged_user: Default::default(),
            settings: Default::default(),
            sidebar: Default::default(),
        }
    }

    pub fn update_state(&mut self, message: AppAction) -> Vec<AppEvent> {
        match message {
            AppAction::Start if !self.started => {
                self.started = true;
                vec![AppEvent::Started]
            }
            AppAction::ShowNotification(c) => vec![AppEvent::NotificationShown(c)],
            AppAction::ViewNowPlaying => vec![AppEvent::NowPlayingShown],
            AppAction::Raise => vec![AppEvent::Raised],
            AppAction::QueueSelection => {
                self.playback.queue(self.selection.take_selection());
                vec![
                    SelectionEvent::SelectionModeChanged(false).into(),
                    PlaybackEvent::PlaylistChanged.into(),
                ]
            }
            AppAction::DequeueSelection => {
                let tracks: Vec<String> = self
                    .selection
                    .take_selection()
                    .into_iter()
                    .map(|s| s.id)
                    .collect();
                self.playback.dequeue(&tracks);

                vec![
                    SelectionEvent::SelectionModeChanged(false).into(),
                    PlaybackEvent::PlaylistChanged.into(),
                ]
            }
            AppAction::MoveDownSelection => {
                let mut selection = self.selection.peek_selection();
                let playback = &mut self.playback;
                selection
                    .next()
                    .and_then(|song| playback.move_down(&song.id))
                    .map(|_| vec![PlaybackEvent::PlaylistChanged.into()])
                    .unwrap_or_else(Vec::new)
            }
            AppAction::MoveUpSelection => {
                let mut selection = self.selection.peek_selection();
                let playback = &mut self.playback;
                selection
                    .next()
                    .and_then(|song| playback.move_up(&song.id))
                    .map(|_| vec![PlaybackEvent::PlaylistChanged.into()])
                    .unwrap_or_else(Vec::new)
            }
            AppAction::SaveSelection => {
                let tracks = self.selection.take_selection();
                let mut events: Vec<AppEvent> = forward_action(
                    BrowserAction::SaveTracks(tracks),
                    self.browser.home_state_mut().unwrap(),
                );
                events.push(SelectionEvent::SelectionModeChanged(false).into());
                events
            }
            AppAction::UnsaveSelection => {
                let tracks: Vec<String> = self
                    .selection
                    .take_selection()
                    .into_iter()
                    .map(|s| s.id)
                    .collect();
                let mut events: Vec<AppEvent> = forward_action(
                    BrowserAction::RemoveSavedTracks(tracks),
                    self.browser.home_state_mut().unwrap(),
                );
                events.push(SelectionEvent::SelectionModeChanged(false).into());
                events
            }
            AppAction::EnableSelection(context) => {
                if let Some(active) = self.selection.set_mode(Some(context)) {
                    vec![SelectionEvent::SelectionModeChanged(active).into()]
                } else {
                    vec![]
                }
            }
            AppAction::CancelSelection => {
                if let Some(active) = self.selection.set_mode(None) {
                    vec![SelectionEvent::SelectionModeChanged(active).into()]
                } else {
                    vec![]
                }
            }
            AppAction::PlaybackAction(a) => forward_action(a, &mut self.playback),
            AppAction::BrowserAction(a) => forward_action(a, &mut self.browser),
            AppAction::SelectionAction(a) => forward_action(a, &mut self.selection),
            AppAction::LoginAction(a) => forward_action(a, &mut self.logged_user),
            AppAction::SettingsAction(a) => forward_action(a, &mut self.settings),
            AppAction::ShowPlaylistCreatedNotification(message, label, id) => {
                vec![AppEvent::PlaylistCreatedNotificationShown(
                    message, label, id,
                )]
            }
            AppAction::SidebarAction(a) => forward_action(a, &mut self.sidebar),
            _ => vec![],
        }
    }
}

fn forward_action<A, E>(
    action: A,
    target: &mut impl UpdatableState<Action = A, Event = E>,
) -> Vec<AppEvent>
where
    A: Clone,
    E: Into<AppEvent>,
{
    target
        .update_with(Cow::Owned(action))
        .into_iter()
        .map(|e| e.into())
        .collect()
}
