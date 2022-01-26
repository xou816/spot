use gio::prelude::*;
use gio::SimpleActionGroup;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::SimpleHeaderBarModel;
use crate::app::components::{labels, PlaylistModel};
use crate::app::models::*;
use crate::app::state::SelectionContext;
use crate::app::state::{
    BrowserAction, BrowserEvent, PlaybackAction, SelectionAction, SelectionState,
};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, ListDiff, ListStore};

pub struct ArtistDetailsModel {
    pub id: String,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl ArtistDetailsModel {
    pub fn new(id: String, app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            id,
            app_model,
            dispatcher,
        }
    }

    pub fn get_artist_name(&self) -> Option<impl Deref<Target = String> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser.artist_state(&self.id)?.artist.as_ref())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumModel>> + '_> {
        self.app_model
            .map_state_opt(|s| Some(&s.browser.artist_state(&self.id)?.albums))
    }

    pub fn load_artist_details(&self, id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.get_artist(&id)
                    .await
                    .map(|artist| BrowserAction::SetArtistDetails(Box::new(artist)).into())
            });
    }

    pub fn open_album(&self, id: &str) {
        self.dispatcher
            .dispatch(AppAction::ViewAlbum(id.to_string()));
    }

    pub fn load_more(&self) -> Option<()> {
        let api = self.app_model.get_spotify();
        let state = self.app_model.get_state();
        let next_page = &state.browser.artist_state(&self.id)?.next_page;

        let id = next_page.data.clone();
        let batch_size = next_page.batch_size;
        let offset = next_page.next_offset?;

        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.get_artist_albums(&id, offset, batch_size)
                    .await
                    .map(|albums| BrowserAction::AppendArtistReleases(albums).into())
            });

        Some(())
    }
}

impl PlaylistModel for ArtistDetailsModel {
    fn song_list_model(&self) -> SongListModel {
        self.app_model
            .get_state()
            .browser
            .artist_state(&self.id)
            .expect("illegal attempt to read artist_state")
            .top_tracks
            .clone()
    }

    fn current_song_id(&self) -> Option<String> {
        self.app_model.get_state().playback.current_song_id()
    }

    fn play_song_at(&self, _pos: usize, id: &str) {
        let tracks: Vec<SongDescription> = self.song_list_model().collect();
        self.dispatcher
            .dispatch(PlaybackAction::Queue(tracks).into());
        self.dispatcher
            .dispatch(PlaybackAction::Load(id.to_string()).into());
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let song = self.song_list_model().get(id)?;
        let song = song.description();

        let group = SimpleActionGroup::new();

        for view_artist in song.make_artist_actions(self.dispatcher.box_clone(), None) {
            group.add_action(&view_artist);
        }
        group.add_action(&song.make_album_action(self.dispatcher.box_clone(), None));
        group.add_action(&song.make_link_action(None));
        group.add_action(&song.make_queue_action(self.dispatcher.box_clone(), None));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let song = self.song_list_model().get(id)?;
        let song = song.description();

        let menu = gio::Menu::new();
        menu.append(Some(&*labels::VIEW_ALBUM), Some("song.view_album"));
        for artist in song.artists.iter().filter(|a| self.id != a.id) {
            menu.append(
                Some(&labels::more_from_label(&artist.name)),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some(&*labels::COPY_LINK), Some("song.copy_link"));
        menu.append(Some(&*labels::ADD_TO_QUEUE), Some("song.queue"));
        Some(menu.upcast())
    }

    fn select_song(&self, id: &str) {
        let song = self.song_list_model().get(id);
        if let Some(song) = song {
            self.dispatcher
                .dispatch(SelectionAction::Select(vec![song.into_description()]).into());
        }
    }

    fn deselect_song(&self, id: &str) {
        self.dispatcher
            .dispatch(SelectionAction::Deselect(vec![id.to_string()]).into());
    }

    fn enable_selection(&self) -> bool {
        self.dispatcher
            .dispatch(AppAction::EnableSelection(SelectionContext::Default));
        true
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        Some(Box::new(self.app_model.map_state(|s| &s.selection)))
    }
}

impl SimpleHeaderBarModel for ArtistDetailsModel {
    fn title(&self) -> Option<String> {
        Some(self.get_artist_name()?.clone())
    }

    fn title_updated(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::BrowserEvent(BrowserEvent::ArtistDetailsUpdated(_))
        )
    }

    fn selection_context(&self) -> Option<&SelectionContext> {
        Some(&SelectionContext::Default)
    }

    fn select_all(&self) {
        let songs: Vec<SongDescription> = self.song_list_model().collect();
        self.dispatcher
            .dispatch(SelectionAction::Select(songs).into());
    }
}
