use crate::app::credentials;
use crate::app::state::*;
use crate::backend::api::SpotifyApiClient;
use ref_filter_map::*;
use std::cell::{Ref, RefCell};
use std::sync::Arc;

pub struct AppServices {
    pub spotify_api: Arc<dyn SpotifyApiClient + Send + Sync>,
}

pub struct AppModel {
    state: RefCell<AppState>,
    services: AppServices,
}

impl AppModel {
    pub fn new(state: AppState, spotify_api: Arc<dyn SpotifyApiClient + Send + Sync>) -> Self {
        let services = AppServices { spotify_api };
        let state = RefCell::new(state);
        Self { state, services }
    }

    pub fn get_spotify(&self) -> Arc<dyn SpotifyApiClient + Send + Sync> {
        Arc::clone(&self.services.spotify_api)
    }

    pub fn get_state(&self) -> Ref<'_, AppState> {
        self.state.borrow()
    }

    pub fn map_state<T: 'static, F: FnOnce(&AppState) -> &T>(&self, map: F) -> Ref<'_, T> {
        Ref::map(self.state.borrow(), map)
    }

    pub fn map_state_opt<T: 'static, F: FnOnce(&AppState) -> Option<&T>>(
        &self,
        map: F,
    ) -> Option<Ref<'_, T>> {
        ref_filter_map(self.state.borrow(), map)
    }

    pub fn update_state(&self, message: AppAction) -> Vec<AppEvent> {
        match message {
            AppAction::LoginSuccess(ref creds) => {
                credentials::save_credentials(creds.clone()).expect("could not save credentials");
                self.services.spotify_api.update_credentials(creds.clone());
            }
            _ => {}
        }

        let mut state = self.state.borrow_mut();
        state.update_state(message)
    }
}
