use crate::api::clear_user_cache;
use crate::app::credentials;
use crate::app::models::{PlaylistDescription, PlaylistSummary};
use crate::app::state::{LoginAction, PlaybackAction};
use crate::app::{ActionDispatcher, AppModel};
use std::ops::Deref;
use std::rc::Rc;

pub struct UserMenuModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl UserMenuModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn username(&self) -> Option<impl Deref<Target = String> + '_> {
        self.app_model
            .map_state_opt(|s| s.logged_user.user.as_ref())
    }

    pub fn logout(&self) {
        self.dispatcher.dispatch(PlaybackAction::Stop.into());
        self.dispatcher.dispatch_async(Box::pin(async {
            let _ = credentials::logout();
            let _ = clear_user_cache().await;
            Some(LoginAction::Logout.into())
        }));
    }

    pub fn fetch_user_playlists(&self) {
        let api = self.app_model.get_spotify();
        if let Some(current_user) = self.username() {
            let current_user = current_user.clone();
            self.dispatcher
                .call_spotify_and_dispatch(move || async move {
                    api.get_saved_playlists(0, 30).await.map(|playlists| {
                        let summaries = playlists
                            .into_iter()
                            .filter(|p| p.owner.id == current_user)
                            .map(|PlaylistDescription { id, title, .. }| PlaylistSummary {
                                id,
                                title,
                            })
                            .collect();
                        LoginAction::SetUserPlaylists(summaries).into()
                    })
                });
        }
    }
}
