use std::rc::Rc;
use std::ops::Deref;
use std::cell::{Ref};

use crate::app::{AppModel, BrowserAction, ActionDispatcher, ListStore};
use crate::app::dispatch::Worker;
use crate::app::models::*;
use crate::app::state::{ScreenName, LibraryState};
use super::Browser;

pub struct BrowserFactory {
    worker: Worker,
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl BrowserFactory {

    pub fn new(
        worker: Worker,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>) -> Self {

        Self { worker, app_model, dispatcher }
    }

    pub fn make_browser(&self) -> Browser {
        let model = BrowserModel::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Browser::new(self.worker.clone(), model)
    }
}

pub struct BrowserModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
    batch_size: u32
}

impl BrowserModel {

    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher, batch_size: 20 }
    }

    fn state(&self) -> Option<Ref<'_, LibraryState>> {
        self.app_model.map_state_opt(|s| s.browser_state.library_state())
    }

    pub fn get_list_store(&self) -> Option<impl Deref<Target = ListStore<AlbumModel>> + '_> {
        Some(Ref::map(self.state()?, |s| &s.albums))
    }

    pub fn refresh_saved_albums(&self) {
        let api = self.app_model.get_spotify();
        let batch_size = self.batch_size;

        self.dispatcher.dispatch_async(Box::pin(async move {
            let albums = api.get_saved_albums(0, batch_size).await?;
            Some(BrowserAction::SetContent(albums).into())
        }));
    }


    pub fn load_more_albums(&self) {
        let api = self.app_model.get_spotify();
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
    }
}
