use std::rc::Rc;
use std::ops::Deref;
use std::cell::{RefCell, Ref};
use ref_filter_map::*;

use crate::app::{AppModel, AppAction, BrowserAction, ActionDispatcher, ListStore};
use crate::app::dispatch::Worker;
use crate::app::models::*;
use crate::app::backend::api::SpotifyApiClient;
use crate::app::state::{ScreenName, LibraryState};
use super::Browser;

pub struct BrowserFactory {
    worker: Worker,
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl BrowserFactory {

    pub fn new(
        worker: Worker,
        app_model: Rc<RefCell<AppModel>>,
        dispatcher: Box<dyn ActionDispatcher>) -> Self {

        Self { worker, app_model, dispatcher }
    }

    pub fn make_browser(&self) -> Browser {
        let model = BrowserModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Browser::new(self.worker.clone(), model)
    }
}

pub struct BrowserModel {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>,
    batch_size: u32
}

impl BrowserModel {

    pub fn new(app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher, batch_size: 20 }
    }

    fn spotify(&self) -> Rc<dyn SpotifyApiClient> {
        Rc::clone(&self.app_model.borrow().services.spotify_api)
    }

    fn state(&self) -> Option<Ref<'_, LibraryState>> {
        ref_filter_map(self.app_model.borrow(), |m| m.state.browser_state.library_state())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumDescription, AlbumModel>> + '_> {
        Some(Ref::map(self.state()?, |s| &s.albums))
    }

    pub fn refresh_saved_albums(&self) {
        let api = self.spotify();
        let batch_size = self.batch_size;

        self.dispatcher.dispatch_async(Box::pin(async move {
            let albums = api.get_saved_albums(0, batch_size).await.unwrap_or(vec![]);
            Some(BrowserAction::SetContent(albums).into())
        }));
    }


    pub fn load_more_albums(&self) {
        let api = self.spotify();
        let page = self.state().map(|s| s.page).unwrap_or(0);
        let offset = page * self.batch_size;
        let batch_size = self.batch_size;

        self.dispatcher.dispatch_async(Box::pin(async move {
            let albums = api.get_saved_albums(offset, batch_size).await.unwrap_or(vec![]);
            Some(BrowserAction::AppendContent(albums).into())
        }));
    }

    pub fn open_album(&self, album_uri: &str) {
        let screen = ScreenName::Details(album_uri.to_owned());
        self.dispatcher.dispatch(BrowserAction::NavigationPush(screen).into());

        let album = self.get_list_store().and_then(|albums| {
            (*albums).iter().find(|&a| a.id.eq(album_uri)).cloned()
        });

        if let Some(album) = album {
            self.dispatcher.dispatch(BrowserAction::SetDetails(album).into());
        }
    }
}
