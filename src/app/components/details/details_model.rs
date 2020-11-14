use std::rc::Rc;
use std::cell::{Ref, RefCell};
use ref_filter_map::*;

use crate::app::AppModel;
use crate::app::models::*;
use crate::app::dispatch::Worker;
use crate::app::components::PlaylistFactory;

use super::*;

pub struct DetailsFactory {
    app_model: Rc<RefCell<AppModel>>,
    worker: Worker,
    playlist_factory: PlaylistFactory
}

impl DetailsFactory {

    pub fn new(app_model: Rc<RefCell<AppModel>>, worker: Worker, playlist_factory: PlaylistFactory) -> Self {
        Self { app_model, worker, playlist_factory }
    }

    pub fn make_details(&self) -> Details {
        let model = DetailsModelImpl::new(Rc::clone(&self.app_model));
        Details::new(Rc::new(model), self.worker.clone(), &self.playlist_factory)
    }
}

struct DetailsModelImpl {
    app_model: Rc<RefCell<AppModel>>
}

impl DetailsModelImpl {

    fn new(app_model: Rc<RefCell<AppModel>>) -> Self {
        Self { app_model }
    }
}

impl DetailsModel for DetailsModelImpl {

    fn get_album_info(&self) -> Option<Ref<'_, AlbumDescription>> {
        ref_filter_map(self.app_model.borrow(), |m| m.state.browser_state.details_state()?.content.as_ref())
    }
}
