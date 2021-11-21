use std::rc::Rc;

use crate::app::components::*;
use crate::app::state::SelectionContext;
use crate::app::{ActionDispatcher, AppModel, Worker};

pub struct ScreenFactory {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
    worker: Worker,
}

impl ScreenFactory {
    pub fn new(
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker,
    ) -> Self {
        Self {
            app_model,
            dispatcher,
            worker,
        }
    }

    pub fn make_library(&self) -> impl ListenerComponent {
        let model = LibraryModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        let screen_model = DefaultScreenModel::new(
            Some(gettext("Library")),
            None,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            Library::new(self.worker.clone(), model),
            Rc::new(screen_model),
        )
    }

    pub fn make_saved_playlists(&self) -> impl ListenerComponent {
        let model =
            SavedPlaylistsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        let screen_model = DefaultScreenModel::new(
            Some(gettext("Playlists")),
            None,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            SavedPlaylists::new(self.worker.clone(), model),
            Rc::new(screen_model),
        )
    }

    pub fn make_now_playing(&self) -> impl ListenerComponent {
        let model = Rc::new(NowPlayingModel::new(
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        let screen_model = SimpleScreenModelWrapper::new(
            Rc::clone(&model),
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            NowPlaying::new(model, self.worker.clone()),
            Rc::new(screen_model),
        )
    }

    pub fn make_saved_tracks(&self) -> impl ListenerComponent {
        let screen_model = DefaultScreenModel::new(
            Some(gettext("Saved tracks")),
            Some(SelectionContext::Default),
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        let model = Rc::new(SavedTracksModel::new(
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        StandardScreen::new(
            SavedTracks::new(model, self.worker.clone()),
            Rc::new(screen_model),
        )
    }

    pub fn make_album_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(DetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        let screen_model = SimpleScreenModelWrapper::new(
            Rc::clone(&model),
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            Details::new(model, self.worker.clone()),
            Rc::new(screen_model),
        )
    }

    pub fn make_search_results(&self) -> impl ListenerComponent {
        let screen_model = DefaultScreenModel::new(
            None,
            None,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        let model =
            SearchResultsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        StandardScreen::new(
            SearchResults::new(model, self.worker.clone()),
            Rc::new(screen_model),
        )
    }

    pub fn make_artist_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(ArtistDetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        let screen_model = SimpleScreenModelWrapper::new(
            Rc::clone(&model),
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            ArtistDetails::new(model, self.worker.clone()),
            Rc::new(screen_model),
        )
    }

    pub fn make_playlist_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(PlaylistDetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        let screen_model = SimpleScreenModelWrapper::new(
            Rc::clone(&model),
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            PlaylistDetails::new(model, self.worker.clone()),
            Rc::new(screen_model),
        )
    }

    pub fn make_user_details(&self, id: String) -> impl ListenerComponent {
        let screen_model = DefaultScreenModel::new(
            None,
            None,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        let model =
            UserDetailsModel::new(id, Rc::clone(&self.app_model), self.dispatcher.box_clone());
        StandardScreen::new(
            UserDetails::new(model, self.worker.clone()),
            Rc::new(screen_model),
        )
    }
}
