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

    pub fn make_now_playing(&self) -> NowPlaying {
        let model = NowPlayingModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        NowPlaying::new(model)
    }

    pub fn make_album_details(&self, id: String) -> Details {
        let model = DetailsModel::new(id, Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Details::new(model, self.worker.clone())
    }

    pub fn make_search_results(&self) -> SearchResults {
        let model =
            SearchResultsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        SearchResults::new(model, self.worker.clone())
    }

    pub fn make_artist_details(&self, id: String) -> ArtistDetails {
        let model =
            ArtistDetailsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        ArtistDetails::new(id, model, self.worker.clone())
    }

    pub fn make_playlist_details(&self, id: String) -> PlaylistDetails {
        let model =
            PlaylistDetailsModel::new(id, Rc::clone(&self.app_model), self.dispatcher.box_clone());
        PlaylistDetails::new(model, self.worker.clone())
    }
}
