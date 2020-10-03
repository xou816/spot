use std::rc::{Rc};
use std::cell::{RefCell};

use crate::app::{AppModel, AbstractDispatcher, AppAction};
use crate::backend::api::SpotifyApi;
use crate::app::components::browser::{BrowserModel, VecAlbumDescriptionFuture, PlayAlbumFuture};

pub struct BrowserModelImpl {
    app_model: Rc<RefCell<AppModel>>,
    api: Rc<SpotifyApi>
}

impl BrowserModelImpl {

    pub fn new(app_model: Rc<RefCell<AppModel>>, api: Rc<SpotifyApi>) -> Self {
        BrowserModelImpl { app_model, api }
    }

    fn api_token(&self) -> Option<String> {
        if let Some(token) = &self.app_model.borrow().state.api_token {
            Some(token.clone())
        } else {
            None
        }
    }

    fn dispatcher(&self) -> Rc<dyn AbstractDispatcher<AppAction>> {
        Rc::clone(&self.app_model.borrow().dispatcher)
    }
}

impl BrowserModel for BrowserModelImpl {


    fn get_saved_albums(&self) -> VecAlbumDescriptionFuture {
        let api = Rc::clone(&self.api);
        let token = self.api_token();
        Box::pin(async move {
            if let Some(token) = token {
                api.get_saved_albums(&token).await.unwrap_or(vec![])
            } else {
                vec![]
            }
        })
    }

    fn play_album(&self, album_uri: &str) -> PlayAlbumFuture {
        let api = Rc::clone(&self.api);
        let dispatcher = self.dispatcher();
        let uri = String::from(album_uri);
        let token = self.api_token();
        Box::pin(async move {
            if let Some(token) = token {
                if let Some(songs) = api.get_album(&token, &uri).await {
                    let first = songs[0].uri.clone();
                    dispatcher.dispatch(AppAction::LoadPlaylist(songs));
                    dispatcher.dispatch(AppAction::Load(first));
                }
            }
        })
    }
}
