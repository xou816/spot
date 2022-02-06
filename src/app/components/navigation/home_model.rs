use crate::app::{ActionDispatcher, AppModel, BrowserAction};
use crate::AppAction;
use gettextrs::*;
use std::rc::Rc;

pub struct HomeModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl HomeModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn create_new_playlist(&self, name: String, user_id: String) {
        let api = self.app_model.get_spotify();
        self.dispatcher
            .call_spotify_and_dispatch_many(move || async move {
                api.create_new_playlist(name.as_str(), user_id.as_str())
                    .await
                    .map(|p| {
                        vec![
                            BrowserAction::PrependPlaylistsContent(vec![p]).into(),
                            AppAction::ShowNotification(gettext!(
                                // translators: This is a notification that pop ups when a new playlist is created. It includes the name of that playlist.
                                "New playlist '{}' created.",
                                name
                            )),
                        ]
                    })
            })
    }
}