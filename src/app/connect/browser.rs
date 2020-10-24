use std::rc::{Rc};
use std::cell::{RefCell};

use crate::app::{AppModel, AppAction};
use crate::app::components::browser::{BrowserModel, VecAlbumDescriptionFuture, PlayAlbumFuture};

pub struct BrowserModelImpl {
    app_model: Rc<RefCell<AppModel>>
}

impl BrowserModelImpl {

    pub fn new(app_model: Rc<RefCell<AppModel>>) -> Self {
        BrowserModelImpl { app_model }
    }
}

impl BrowserModel for BrowserModelImpl {


    fn get_saved_albums(&self) -> VecAlbumDescriptionFuture {
        let app_model = self.app_model.borrow();
        let api = Rc::clone(&app_model.services.spotify_api);

        Box::pin(async move {
            api.get_saved_albums().await.unwrap_or(vec![])
        })
    }

    fn play_album(&self, album_uri: &str) -> PlayAlbumFuture {
        let app_model = self.app_model.borrow();
        let dispatcher = app_model.dispatcher.box_clone();
        let api = Rc::clone(&app_model.services.spotify_api);
        let uri = String::from(album_uri);

        Box::pin(async move {
            if let Some(songs) = api.get_album(&uri).await {
                let first = songs[0].uri.clone();
                dispatcher.dispatch(AppAction::LoadPlaylist(songs));
                dispatcher.dispatch(AppAction::Load(first));
            }
        })
    }
}
