use gio::prelude::*;
use gio::{ActionMapExt, SimpleActionGroup};
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{
    handle_error, labels, PlaylistModel, SelectionTool, SelectionToolsModel,
};
use crate::app::models::SongModel;
use crate::app::state::{
    PlaybackAction, PlaybackEvent, PlaybackState, PlaylistSource, SelectionAction,
    SelectionContext, SelectionState,
};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, AppState, ListDiff};

pub struct NowPlayingModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl NowPlayingModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> Ref<'_, AppState> {
        self.app_model.get_state()
    }

    fn queue(&self) -> Ref<'_, PlaybackState> {
        Ref::map(self.state(), |s| &s.playback)
    }

    pub fn toggle_shuffle(&self) {
        self.dispatcher
            .dispatch(PlaybackAction::ToggleShuffle.into());
    }

    pub fn load_more_if_needed(&self) -> Option<()> {
        let queue = self.queue();
        if queue.position() < PlaybackState::max_size() - 1 {
            return None;
        }

        let api = self.app_model.get_spotify();
        let pagination = queue.pagination.as_ref()?;
        let batch_size = pagination.batch_size as u32;
        let next_offset = pagination.next_offset? as u32;

        if let PlaylistSource::Playlist(id) = &pagination.data {
            let id = id.clone();
            self.dispatcher.dispatch_async(Box::pin(async move {
                match api.get_playlist_tracks(&id, next_offset, batch_size).await {
                    Ok(tracks) => Some(PlaybackAction::QueueMany(tracks).into()),
                    Err(err) => handle_error(err),
                }
            }));
        }

        Some(())
    }
}

impl PlaylistModel for NowPlayingModel {
    fn current_song_id(&self) -> Option<String> {
        self.queue().current_song_id.clone()
    }

    fn play_song(&self, id: &str) {
        self.dispatcher
            .dispatch(PlaybackAction::Load(id.to_string()).into());
    }

    fn diff_for_event(&self, event: &AppEvent) -> Option<ListDiff<SongModel>> {
        if matches!(
            event,
            AppEvent::PlaybackEvent(PlaybackEvent::PlaylistChanged)
        ) {
            Some(ListDiff::Set(
                self.queue()
                    .songs()
                    .enumerate()
                    .map(|(i, s)| s.to_song_model(i))
                    .collect(),
            ))
        } else {
            None
        }
    }

    fn autoscroll_to_playing(&self) -> bool {
        true
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let queue = self.queue();
        let song = queue.song(id)?;
        let group = SimpleActionGroup::new();

        for view_artist in song.make_artist_actions(self.dispatcher.box_clone(), None) {
            group.add_action(&view_artist);
        }
        group.add_action(&song.make_album_action(self.dispatcher.box_clone(), None));
        group.add_action(&song.make_link_action(None));
        group.add_action(&song.make_dequeue_action(self.dispatcher.box_clone(), None));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let queue = self.queue();
        let song = queue.song(id)?;

        let menu = gio::Menu::new();
        menu.append(Some(&*labels::VIEW_ALBUM), Some("song.view_album"));
        for artist in song.artists.iter() {
            menu.append(
                Some(&format!("{} {}", *labels::MORE_FROM, artist.name)),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some(&*labels::COPY_LINK), Some("song.copy_link"));
        menu.append(Some(&*labels::REMOVE_FROM_QUEUE), Some("song.dequeue"));

        Some(menu.upcast())
    }

    fn select_song(&self, id: &str) {
        let queue = self.queue();
        if let Some(song) = queue.song(id) {
            self.dispatcher
                .dispatch(SelectionAction::Select(vec![song.clone()]).into());
        }
    }

    fn deselect_song(&self, id: &str) {
        self.dispatcher
            .dispatch(SelectionAction::Deselect(vec![id.to_string()]).into());
    }

    fn enable_selection(&self) -> bool {
        self.dispatcher
            .dispatch(AppAction::ChangeSelectionMode(true));
        true
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        let selection = self
            .app_model
            .map_state_opt(|s| Some(&s.selection))
            .filter(|s| s.context == SelectionContext::Queue)?;
        Some(Box::new(selection))
    }
}

impl SelectionToolsModel for NowPlayingModel {
    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        let selection = self
            .app_model
            .map_state_opt(|s| Some(&s.selection))
            .filter(|s| s.context == SelectionContext::Queue)?;
        Some(Box::new(selection))
    }

    fn handle_tool_activated(&self, selection: &SelectionState, tool: &SelectionTool) {
        let action = match (tool, tool.default_action()) {
            (_, Some(action)) => Some(action),
            (SelectionTool::SelectAll, None) => {
                let queue = self.queue();
                let all_selected = selection.all_selected(queue.songs().map(|s| &s.id));
                Some(
                    if all_selected {
                        SelectionAction::Deselect(queue.songs().map(|s| &s.id).cloned().collect())
                    } else {
                        SelectionAction::Select(queue.songs().cloned().collect())
                    }
                    .into(),
                )
            }
            _ => None,
        };
        if let Some(action) = action {
            self.dispatcher.dispatch(action);
        }
    }
}
