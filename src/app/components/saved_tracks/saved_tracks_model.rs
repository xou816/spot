use gio::prelude::*;
use gio::SimpleActionGroup;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crate::app::components::{labels, PlaylistModel, SelectionTool, SelectionToolsModel};
use crate::app::models::*;
use crate::app::state::{PlaybackAction, SelectionAction, SelectionContext, SelectionState};
use crate::app::{
    ActionDispatcher, AppAction, AppEvent, AppModel, BatchQuery, BrowserAction, BrowserEvent,
    ListDiff, SongsSource,
};
use crate::{api::SpotifyApiClient, app::components::SimpleSelectionTool};

pub struct SavedTracksModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SavedTracksModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn songs(&self) -> Option<impl Deref<Target = SongList> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.home_state()?.saved_tracks))
    }

    pub fn load_initial(&self) {
        let query = BatchQuery {
            source: SongsSource::SavedTracks,
            batch: Batch::first_of_size(50),
        };

        self.load_batch(query);
    }

    pub fn load_more(&self) -> Option<()> {
        let last_batch = self.songs()?.last_batch()?;
        let query = BatchQuery {
            source: SongsSource::SavedTracks,
            batch: last_batch,
        };

        self.load_batch(query.next()?);
        Some(())
    }

    fn load_batch(&self, query: BatchQuery) {
        let loader = self.app_model.get_batch_loader();

        self.dispatcher.dispatch_async(Box::pin(async move {
            let action = loader
                .query(query, |song_batch| {
                    BrowserAction::AppendSavedTracks(Box::new(song_batch)).into()
                })
                .await;
            Some(action)
        }));
    }
}

impl PlaylistModel for SavedTracksModel {
    fn current_song_id(&self) -> Option<String> {
        self.app_model
            .get_state()
            .playback
            .current_song_id()
            .cloned()
    }

    fn play_song_at(&self, pos: usize, id: &str) {
        let source = SongsSource::SavedTracks;
        let batch = self.songs().and_then(|songs| songs.song_batch_for(pos));
        if let Some(batch) = batch {
            self.dispatcher
                .dispatch(PlaybackAction::LoadPagedSongs(source, batch).into());
            self.dispatcher
                .dispatch(PlaybackAction::Load(id.to_string()).into());
        }
    }

    fn diff_for_event(&self, event: &AppEvent) -> Option<ListDiff<SongModel>> {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::SavedTracksAppended(i)) => {
                let songs = self.songs()?;
                Some(ListDiff::Append(
                    songs.iter().skip(*i).map(|s| s.into()).collect(),
                ))
            }
            _ => None,
        }
    }

    fn autoscroll_to_playing(&self) -> bool {
        true
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let songs = self.songs()?;
        let song = songs.get(id)?;

        let group = SimpleActionGroup::new();

        for view_artist in song.make_artist_actions(self.dispatcher.box_clone(), None) {
            group.add_action(&view_artist);
        }
        group.add_action(&song.make_album_action(self.dispatcher.box_clone(), None));
        group.add_action(&song.make_link_action(None));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let songs = self.songs()?;
        let song = songs.get(id)?;

        let menu = gio::Menu::new();
        menu.append(Some(&*labels::VIEW_ALBUM), Some("song.view_album"));
        for artist in song.artists.iter() {
            menu.append(
                Some(&format!(
                    "{} {}",
                    *labels::MORE_FROM,
                    glib::markup_escape_text(&artist.name)
                )),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some(&*labels::COPY_LINK), Some("song.copy_link"));

        Some(menu.upcast())
    }

    fn select_song(&self, id: &str) {
        let song = self.songs().and_then(|s| s.get(id).cloned());
        if let Some(song) = song {
            self.dispatcher
                .dispatch(SelectionAction::Select(vec![song]).into());
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

impl SelectionToolsModel for SavedTracksModel {
    fn dispatcher(&self) -> Box<dyn ActionDispatcher> {
        self.dispatcher.box_clone()
    }

    fn spotify_client(&self) -> Arc<dyn SpotifyApiClient + Send + Sync> {
        self.app_model.get_spotify()
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        let selection = self
            .app_model
            .map_state_opt(|s| Some(&s.selection))
            .filter(|s| s.context == SelectionContext::Queue)?;
        Some(Box::new(selection))
    }

    fn tools_visible(&self, _: &SelectionState) -> Vec<SelectionTool> {
        vec![SelectionTool::Simple(SimpleSelectionTool::SelectAll)]
    }

    fn handle_tool_activated(&self, selection: &SelectionState, tool: &SelectionTool) {
        match tool {
            SelectionTool::Simple(SimpleSelectionTool::SelectAll) => {
                if let Some(songs) = self.songs() {
                    let vec = songs.iter().collect::<Vec<&SongDescription>>();
                    self.handle_select_all_tool_borrowed(selection, &vec[..]);
                }
            }
            _ => self.default_handle_tool_activated(selection, tool),
        };
    }
}
