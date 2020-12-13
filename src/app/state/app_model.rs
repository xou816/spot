use std::rc::Rc;
use std::cell::{Ref, RefCell};
use ref_filter_map::*;
use crate::app::state::*;
use crate::app::credentials;
use crate::backend::api::SpotifyApiClient;


pub struct AppServices {
    pub spotify_api: Rc<dyn SpotifyApiClient>
}

pub struct AppModel {
    state: RefCell<AppState>,
    pub services: AppServices
}

impl AppModel {

    pub fn new(
        state: AppState,
        spotify_api: Rc<dyn SpotifyApiClient>) -> Self {

        let services = AppServices { spotify_api };
        let state = RefCell::new(state);
        Self { state, services }
    }

    pub fn get_state(&self) -> Ref<'_, AppState> {
        self.state.borrow()
    }

    pub fn map_state<T: 'static, F: FnOnce(&AppState) -> &T>(&self, map: F) -> Ref<'_, T> {
        Ref::map(self.state.borrow(), map)
    }

    pub fn map_state_opt<T: 'static, F: FnOnce(&AppState) -> Option<&T>>(&self, map: F) -> Option<Ref<'_, T>> {
        ref_filter_map(self.state.borrow(), map)
    }

    pub fn update_state(&self, message: AppAction) -> Vec<AppEvent> {
        match message {
            AppAction::LoginSuccess(ref creds) => {
                credentials::save_credentials(creds.clone()).expect("could not save credentials");
                self.services.spotify_api.update_token(&creds.token[..]);
            },
            _ => {}
        }

        let mut state = self.state.borrow_mut();
        state.update_state(message)
    }
}

