use std::rc::Rc;

use crate::app::components::*;
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

    pub fn make_library(&self) -> Library {
        let model = LibraryModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Library::new(self.worker.clone(), model)
    }

    pub fn make_saved_playlists(&self) -> SavedPlaylists {
        let model =
            SavedPlaylistsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        SavedPlaylists::new(self.worker.clone(), model)
    }

    pub fn make_now_playing(&self) -> impl ListenerComponent {
        let model = Rc::new(NowPlayingModel::new(
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        SelectionTools::new(NowPlaying::new(Rc::clone(&model)), model)
    }

    pub fn make_album_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(DetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        SelectionTools::new(Details::new(Rc::clone(&model), self.worker.clone()), model)
    }

    pub fn make_album_info(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(AlbumInfoModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        SelectionTools::new(Info::new(Rc::clone(&model)), model)
    }

    pub fn make_search_results(&self) -> SearchResults {
        let model =
            SearchResultsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        SearchResults::new(model, self.worker.clone())
    }

    pub fn make_artist_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(ArtistDetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        SelectionTools::new(
            ArtistDetails::new(Rc::clone(&model), self.worker.clone()),
            model,
        )
    }

    pub fn make_playlist_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(PlaylistDetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        SelectionTools::new(
            PlaylistDetails::new(Rc::clone(&model), self.worker.clone()),
            model,
        )
    }

    pub fn make_user_details(&self, id: String) -> UserDetails {
        let model =
            UserDetailsModel::new(id, Rc::clone(&self.app_model), self.dispatcher.box_clone());
        UserDetails::new(model, self.worker.clone())
    }
}
