use std::rc::{Rc};
use std::cell::{RefCell, Ref};

use crate::app::{AppModel, AppAction, BrowserAction, ActionDispatcher};
use crate::app::dispatch::Worker;
use crate::app::models::*;
use crate::app::components::browser::{Browser, BrowserModel};
use crate::app::backend::api::SpotifyApiClient;


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

    pub fn make_browser(&self, flowbox: gtk::FlowBox, scroll_window: gtk::ScrolledWindow) -> Browser {
        let model = BrowserModelImpl::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Browser::new(flowbox, scroll_window, self.worker.clone(), Rc::new(model))
    }
}

pub struct BrowserModelImpl {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>,
    batch_size: u32
}

impl BrowserModelImpl {

    pub fn new(app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        BrowserModelImpl { app_model, dispatcher, batch_size: 20 }
    }

    fn spotify(&self) -> Rc<dyn SpotifyApiClient> {
        Rc::clone(&self.app_model.borrow().services.spotify_api)
    }
}

impl BrowserModel for BrowserModelImpl {

    fn get_saved_albums(&self) -> Ref<'_, Vec<AlbumDescription>> {
        Ref::map(self.app_model.borrow(), |m| &m.state.browser_state.albums)
    }

    fn refresh_saved_albums(&self) {
        let api = self.spotify();
        let batch_size = self.batch_size;

        self.dispatcher.dispatch_async(Box::pin(async move {
            let albums = api.get_saved_albums(0, batch_size).await.unwrap_or(vec![]);
            Some(BrowserAction::SetContent(albums).into())
        }));
    }


    fn load_more_albums(&self) {
        let app_model = self.app_model.borrow();
        let api = self.spotify();
        let offset = app_model.state.browser_state.page * self.batch_size;
        let batch_size = self.batch_size;

        self.dispatcher.dispatch_async(Box::pin(async move {
            let albums = api.get_saved_albums(offset, batch_size).await.unwrap_or(vec![]);
            Some(BrowserAction::AppendContent(albums).into())
        }));
    }

    fn play_album(&self, album_uri: &str) {
        let app_model = self.app_model.borrow();
        let api = Rc::clone(&app_model.services.spotify_api);
        let uri = String::from(album_uri);

        self.dispatcher.dispatch_many_async(Box::pin(async move {
            if let Some(songs) = api.get_album(&uri).await {
                let first_song = songs[0].uri.clone();
                vec![AppAction::LoadPlaylist(songs), AppAction::Load(first_song)]
            } else {
                vec![]
            }
        }));
    }
}
