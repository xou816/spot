use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::PlaylistFactory;
use crate::app::dispatch::{ActionDispatcher, Worker};
use crate::app::models::*;
use crate::app::state::{BrowserAction, ScreenName};
use crate::app::AppModel;

use super::Details;

pub struct DetailsFactory {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
    worker: Worker,
    playlist_factory: PlaylistFactory,
}

impl DetailsFactory {
    pub fn new(
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker,
        playlist_factory: PlaylistFactory,
    ) -> Self {
        Self {
            app_model,
            dispatcher,
            worker,
            playlist_factory,
        }
    }

    pub fn make_details(&self, id: String) -> Details {
        let model = DetailsModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Details::new(id, model, self.worker.clone(), &self.playlist_factory)
    }
}

pub struct DetailsModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl DetailsModel {
    fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn get_album_info(&self) -> Option<impl Deref<Target = AlbumDescription> + '_> {
        self.app_model
            .map_state_opt(|s| s.browser_state.details_state()?.content.as_ref())
    }

    pub fn load_album_info(&self, id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher.dispatch_async(Box::pin(async move {
            let album = api.get_album(&id).await?;
            Some(BrowserAction::SetDetails(album).into())
        }));
    }

    pub fn view_artist(&self) {
        if let Some(album) = self.get_album_info() {
            let artist = &album.artists.first().unwrap().id;
            self.dispatcher.dispatch(
                BrowserAction::NavigationPush(ScreenName::Artist(artist.to_owned())).into(),
            );
        }
    }
}
