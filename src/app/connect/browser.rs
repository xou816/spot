use std::rc::{Rc};
use std::cell::{RefCell, Ref};
use futures::future::LocalBoxFuture;

use crate::app::{AppModel, AppAction, BrowserAction};
use crate::app::models::*;
use crate::app::components::browser::{BrowserModel};

pub struct BrowserModelImpl {
    app_model: Rc<RefCell<AppModel>>,
    batch_size: u32
}

impl BrowserModelImpl {

    pub fn new(app_model: Rc<RefCell<AppModel>>) -> Self {
        BrowserModelImpl { app_model, batch_size: 20 }
    }
}

impl BrowserModel for BrowserModelImpl {

    fn get_saved_albums(&self) -> Ref<'_, Vec<AlbumDescription>> {
        Ref::map(self.app_model.borrow(), |m| &m.state.browser_state.albums)
    }

    fn refresh_saved_albums(&self) -> LocalBoxFuture<()> {
        let app_model = self.app_model.borrow();
        let dispatcher = app_model.dispatcher.box_clone();
        let api = Rc::clone(&app_model.services.spotify_api);

        Box::pin(async move {
            let albums = api.get_saved_albums(0, self.batch_size).await.unwrap_or(vec![]);
            let action = AppAction::BrowserAction(BrowserAction::SetContent(albums));
            dispatcher.dispatch(action);
        })
    }


    fn load_more_albums(&self) -> LocalBoxFuture<()> {
        let app_model = self.app_model.borrow();
        let dispatcher = app_model.dispatcher.box_clone();
        let api = Rc::clone(&app_model.services.spotify_api);
        let offset = app_model.state.browser_state.page * self.batch_size;

        Box::pin(async move {
            let albums = api.get_saved_albums(offset, self.batch_size).await.unwrap_or(vec![]);
            let action = AppAction::BrowserAction(BrowserAction::AppendContent(albums));
            dispatcher.dispatch(action);
        })
    }

    fn play_album(&self, album_uri: &str) -> LocalBoxFuture<()> {
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
