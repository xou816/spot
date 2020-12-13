use std::rc::Rc;
use std::ops::Deref;

use crate::app::AppModel;
use crate::app::models::*;
use crate::app::dispatch::Worker;
use crate::app::components::PlaylistFactory;

use super::Details;

pub struct DetailsFactory {
    app_model: Rc<AppModel>,
    worker: Worker,
    playlist_factory: PlaylistFactory
}

impl DetailsFactory {

    pub fn new(app_model: Rc<AppModel>, worker: Worker, playlist_factory: PlaylistFactory) -> Self {
        Self { app_model, worker, playlist_factory }
    }

    pub fn make_details(&self) -> Details {
        let model = DetailsModel::new(Rc::clone(&self.app_model));
        Details::new(model, self.worker.clone(), &self.playlist_factory)
    }
}

pub struct DetailsModel {
    app_model: Rc<AppModel>
}

impl DetailsModel {

    fn new(app_model: Rc<AppModel>) -> Self {
        Self { app_model }
    }

    pub fn get_album_info(&self) -> Option<impl Deref<Target = AlbumDescription> + '_> {
        self.app_model.map_state_opt(|s| s.browser_state.details_state()?.content.as_ref())
    }
}
