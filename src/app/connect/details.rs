use std::rc::{Rc};
use std::cell::{Ref, RefCell};

use crate::app::{AppModel};
use crate::app::models::*;
use crate::app::components::{Details, DetailsModel};
use super::PlaylistFactory;

pub struct DetailsFactory {
    app_model: Rc<RefCell<AppModel>>,
    playlist_factory: PlaylistFactory
}

impl DetailsFactory {

    pub fn new(app_model: Rc<RefCell<AppModel>>, playlist_factory: PlaylistFactory) -> Self {
        Self { app_model, playlist_factory }
    }

    pub fn make_details(&self) -> Details {
        let model = DetailsModelImpl::new(Rc::clone(&self.app_model));
        Details::new(Rc::new(model), &self.playlist_factory)
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
        let app_model = self.app_model.borrow();

        if app_model.state.browser_state.details_state().and_then(|s| s.content.as_ref()).is_some() {
            Some(Ref::map(app_model, |m| m.state.browser_state.details_state().unwrap().content.as_ref().unwrap()))
        } else {
            None
        }
    }
}
