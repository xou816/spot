use crate::app::state::SidebarAction;
use crate::app::{ActionDispatcher, AppModel, BrowserAction};
use crate::AppAction;
use gettextrs::*;
use std::ops::Deref;
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
                            BrowserAction::PrependPlaylistsContent(vec![p.clone()]).into(),
                            AppAction::ShowPlaylistCreatedNotification(
                                // translators: This is a notification that pop ups when a new playlist is created. It includes the name of that playlist.
                                gettext("New playlist created."),
                                name,
                                p.id,
                            ),
                        ]
                    })
            })
    }

    pub fn previously_selected_sidebar_item(&self) -> String {
        let item = self
            .app_model
            .map_state(|s| s.sidebar.get_previously_selected_item());
        item.deref().clone()
    }

    pub fn sidebar_item_activated(&self, item: String, id: i32) {
        self.dispatcher
            .dispatch(AppAction::SidebarAction(SidebarAction::SelectItem(
                item, id,
            )));
    }

    pub fn currently_selected_sidebar_index(&self) -> i32 {
        let i = self
            .app_model
            .map_state(|s| s.sidebar.get_currently_selected_index());
        i.deref().clone()
    }

    pub fn reselect_currently_selected_row(&self, listbox: gtk::ListBox) {
        let to_select = listbox.row_at_index(self.currently_selected_sidebar_index());
        listbox.select_row(to_select.as_ref());
    }
}
