use gio::prelude::*;
use gio::SimpleActionGroup;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{
    labels, DeviceSelectorModel, HeaderBarModel, PlaylistModel, SimpleHeaderBarModel,
    SimpleHeaderBarModelWrapper,
};
use crate::app::models::{SongDescription, SongListModel};
use crate::app::state::Device;
use crate::app::state::{
    PlaybackAction, PlaybackState, SelectionAction, SelectionContext, SelectionState,
};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel};

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

    fn queue(&self) -> impl Deref<Target = PlaybackState> + '_ {
        self.app_model.map_state(|s| &s.playback)
    }

    pub fn load_more(&self) -> Option<()> {
        let queue = self.queue();
        let loader = self.app_model.get_batch_loader();
        let query = queue.next_query()?;
        debug!("next_query = {:?}", &query);

        self.dispatcher.dispatch_async(Box::pin(async move {
            loader
                .query(query, |source, song_batch| {
                    PlaybackAction::LoadPagedSongs(source, song_batch).into()
                })
                .await
        }));

        Some(())
    }

    pub fn to_headerbar_model(self: &Rc<Self>) -> Rc<impl HeaderBarModel> {
        Rc::new(SimpleHeaderBarModelWrapper::new(
            self.clone(),
            self.app_model.clone(),
            self.dispatcher.box_clone(),
        ))
    }

    pub fn device_selector_model(&self) -> DeviceSelectorModel {
        DeviceSelectorModel::new(self.app_model.clone(), self.dispatcher.box_clone())
    }

    fn current_selection_context(&self) -> SelectionContext {
        let state = self.app_model.get_state();
        match state.playback.current_device() {
            Device::Local => SelectionContext::Queue,
            Device::Connect(_) => SelectionContext::ReadOnlyQueue,
        }
    }
}

impl PlaylistModel for NowPlayingModel {
    fn song_list_model(&self) -> SongListModel {
        self.queue().songs().clone()
    }

    fn is_paused(&self) -> bool {
        !self.app_model.get_state().playback.is_playing()
    }

    fn current_song_id(&self) -> Option<String> {
        self.queue().current_song_id()
    }

    fn play_song_at(&self, _pos: usize, id: &str) {
        self.dispatcher
            .dispatch(PlaybackAction::Load(id.to_string()).into());
    }

    fn autoscroll_to_playing(&self) -> bool {
        false // too buggy for now
    }

    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup> {
        let queue = self.queue();
        let song = queue.songs().get(id)?;
        let song = song.description();
        let group = SimpleActionGroup::new();

        for view_artist in song.make_artist_actions(self.dispatcher.box_clone(), None) {
            group.add_action(&view_artist);
        }
        group.add_action(&song.make_album_action(self.dispatcher.box_clone(), None));
        group.add_action(&song.make_link_action(None));
        group.add_action(&song.make_dequeue_action(self.dispatcher.box_clone(), None));
        group.add_action(&song.make_like_action(
            self.dispatcher.box_clone(),
            self.app_model.clone(),
            None,
        ));
        group.add_action(&song.make_unlike_action(
            self.dispatcher.box_clone(),
            self.app_model.clone(),
            None,
        ));

        Some(group.upcast())
    }

    fn menu_for(&self, id: &str) -> Option<gio::MenuModel> {
        let queue = self.queue();
        let song = queue.songs().get(id)?;
        let song = song.description();

        let menu = gio::Menu::new();
        menu.append(Some(&*labels::VIEW_ALBUM), Some("song.view_album"));
        for artist in song.artists.iter() {
            menu.append(
                Some(&labels::more_from_label(&artist.name)),
                Some(&format!("song.view_artist_{}", artist.id)),
            );
        }

        menu.append(Some(&*labels::COPY_LINK), Some("song.copy_link"));
        menu.append(Some(&*labels::REMOVE_FROM_QUEUE), Some("song.dequeue"));
        menu.append(Some(&*labels::ADD_TO_LIBRARY), Some("song.like"));
        menu.append(Some(&*labels::REMOVE_FROM_LIBRARY), Some("song.unlike"));

        Some(menu.upcast())
    }

    fn select_song(&self, id: &str) {
        let queue = self.queue();
        if let Some(song) = queue.songs().get(id) {
            let song = song.description().clone();
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
            .dispatch(AppAction::EnableSelection(self.current_selection_context()));
        true
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        let selection = self.app_model.map_state(|s| &s.selection);
        Some(Box::new(selection))
    }
}

impl SimpleHeaderBarModel for NowPlayingModel {
    fn title(&self) -> Option<String> {
        None
    }

    fn title_updated(&self, _: &AppEvent) -> bool {
        false
    }

    fn selection_context(&self) -> Option<SelectionContext> {
        Some(self.current_selection_context())
    }

    fn select_all(&self) {
        let songs: Vec<SongDescription> = self.queue().songs().collect();
        self.dispatcher
            .dispatch(SelectionAction::Select(songs).into());
    }
}
