use std::rc::Rc;

use crate::app::components::sidebar::{Sidebar, SidebarModel};
use crate::app::components::*;
use crate::app::state::SelectionContext;
use crate::app::{ActionDispatcher, AppModel, Worker};

pub struct ScreenFactory {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
    worker: Worker,
    leaflet: libadwaita::Leaflet,
}

impl ScreenFactory {
    pub fn new(
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker,
        leaflet: libadwaita::Leaflet,
    ) -> Self {
        Self {
            app_model,
            dispatcher,
            worker,
            leaflet,
        }
    }

    pub fn make_library(&self) -> impl ListenerComponent {
        let model = LibraryModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        let screen_model = DefaultHeaderBarModel::new(
            Some(gettext("Library")),
            None,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            Library::new(self.worker.clone(), model),
            &self.leaflet,
            Rc::new(screen_model),
        )
    }

    pub fn make_sidebar(&self, listbox: gtk::ListBox) -> impl ListenerComponent {
        let model = SidebarModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Sidebar::new(listbox, Rc::new(model))
    }

    pub fn make_saved_playlists(&self) -> impl ListenerComponent {
        let model =
            SavedPlaylistsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        let screen_model = DefaultHeaderBarModel::new(
            Some(gettext("Playlists")),
            None,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            SavedPlaylists::new(self.worker.clone(), model),
            &self.leaflet,
            Rc::new(screen_model),
        )
    }

    pub fn make_followed_artists(&self) -> impl ListenerComponent {
        let model =
            FollowedArtistsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        let screen_model = DefaultHeaderBarModel::new(
            Some(gettext("Artists")),
            None,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            FollowedArtists::new(self.worker.clone(), model),
            &self.leaflet,
            Rc::new(screen_model),
        )
    }

    pub fn make_now_playing(&self) -> impl ListenerComponent {
        let model = Rc::new(NowPlayingModel::new(
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        NowPlaying::new(model, self.worker.clone(), &self.leaflet)
    }

    pub fn make_saved_tracks(&self) -> impl ListenerComponent {
        let screen_model = DefaultHeaderBarModel::new(
            Some(gettext("Saved tracks")),
            Some(SelectionContext::SavedTracks),
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        let model = Rc::new(SavedTracksModel::new(
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        StandardScreen::new(
            SavedTracks::new(model, self.worker.clone()),
            &self.leaflet,
            Rc::new(screen_model),
        )
    }

    pub fn make_album_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(DetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        Details::new(model, self.worker.clone(), &self.leaflet)
    }

    pub fn make_search_results(&self) -> impl ListenerComponent {
        let model =
            SearchResultsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        SearchResults::new(model, self.worker.clone(), &self.leaflet)
    }

    pub fn make_artist_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(ArtistDetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        let screen_model = SimpleHeaderBarModelWrapper::new(
            Rc::clone(&model),
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        StandardScreen::new(
            ArtistDetails::new(model, self.worker.clone()),
            &self.leaflet,
            Rc::new(screen_model),
        )
    }

    pub fn make_playlist_details(&self, id: String) -> impl ListenerComponent {
        let model = Rc::new(PlaylistDetailsModel::new(
            id,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        ));
        PlaylistDetails::new(model, self.worker.clone())
    }

    pub fn make_user_details(&self, id: String) -> impl ListenerComponent {
        let screen_model = DefaultHeaderBarModel::new(
            None,
            None,
            Rc::clone(&self.app_model),
            self.dispatcher.box_clone(),
        );
        let model =
            UserDetailsModel::new(id, Rc::clone(&self.app_model), self.dispatcher.box_clone());
        StandardScreen::new(
            UserDetails::new(model, self.worker.clone()),
            &self.leaflet,
            Rc::new(screen_model),
        )
    }
}
