use crate::api::SpotifyApiClient;
use crate::app::{state::*, BatchLoader};
use ref_filter_map::*;
use std::cell::{Ref, RefCell};
use std::sync::Arc;

pub struct AppServices {
    pub spotify_api: Arc<dyn SpotifyApiClient + Send + Sync>,
    pub batch_loader: BatchLoader,
}

// Two purposes: give access to some services to users of the AppModel (shared)
// and give a read only view of the state
pub struct AppModel {
    state: RefCell<AppState>,
    services: AppServices,
}

impl AppModel {
    pub fn new(state: AppState, spotify_api: Arc<dyn SpotifyApiClient + Send + Sync>) -> Self {
        let services = AppServices {
            batch_loader: BatchLoader::new(Arc::clone(&spotify_api)),
            spotify_api,
        };
        let state = RefCell::new(state);
        Self { state, services }
    }

    pub fn get_spotify(&self) -> Arc<dyn SpotifyApiClient + Send + Sync> {
        Arc::clone(&self.services.spotify_api)
    }

    pub fn get_batch_loader(&self) -> BatchLoader {
        self.services.batch_loader.clone()
    }

    // Read only access to the state!
    pub fn get_state(&self) -> Ref<'_, AppState> {
        self.state.borrow()
    }

    // Convenience...
    pub fn map_state<T: 'static, F: FnOnce(&AppState) -> &T>(&self, map: F) -> Ref<'_, T> {
        Ref::map(self.state.borrow(), map)
    }

    // Convenience...
    pub fn map_state_opt<T: 'static, F: FnOnce(&AppState) -> Option<&T>>(
        &self,
        map: F,
    ) -> Option<Ref<'_, T>> {
        ref_filter_map(self.state.borrow(), map)
    }

    pub fn update_state(&self, action: AppAction) -> Vec<AppEvent> {
        // The LoginActions are a bit special, we intercept them to grab the Spotify token
        // and save it in our Arc'd API client
        match &action {
            AppAction::LoginAction(LoginAction::SetLoginSuccess(
                SetLoginSuccessAction::Password(creds),
            )) => {
                self.services.spotify_api.update_token(creds.token.clone());
            }
            AppAction::LoginAction(LoginAction::SetLoginSuccess(
                SetLoginSuccessAction::Token { token, .. },
            )) => {
                self.services.spotify_api.update_token(token.clone());
            }
            AppAction::LoginAction(LoginAction::SetRefreshedToken { token, .. }) => {
                self.services.spotify_api.update_token(token.clone());
            }
            _ => {}
        }

        // And this is the only mutable borrow of our state!
        let mut state = self.state.borrow_mut();
        state.update_state(action)
    }
}
