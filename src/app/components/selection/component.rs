use gettextrs::gettext;
use gtk::prelude::*;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{Component, EventListener};
use crate::app::models::PlaylistSummary;
use crate::app::state::{
    LoginEvent, SelectionAction, SelectionContext, SelectionEvent, SelectionState,
};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, BrowserAction};

use super::widget::{SelectionToolState, SelectionToolbarWidget};

pub struct SelectionToolbarModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SelectionToolbarModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn move_up_selection(&self) {
        self.dispatcher.dispatch(AppAction::MoveUpSelection);
    }

    pub fn move_down_selection(&self) {
        self.dispatcher.dispatch(AppAction::MoveDownSelection);
    }

    pub fn queue_selection(&self) {
        self.dispatcher.dispatch(AppAction::QueueSelection);
    }

    fn dequeue_selection(&self) {
        self.dispatcher.dispatch(AppAction::DequeueSelection);
    }

    pub fn remove_selection(&self) {
        match &self.selection().context {
            SelectionContext::SavedTracks => self.remove_saved_tracks(),
            SelectionContext::Queue => self.dequeue_selection(),
            SelectionContext::EditablePlaylist(id) => self.remove_from_playlist(id),
            _ => {}
        }
    }

    pub fn save_selection(&self) {
        let api = self.app_model.get_spotify();
        let ids: Vec<String> = self
            .selection()
            .peek_selection()
            .map(|s| &s.id)
            .cloned()
            .collect();
        self.dispatcher
            .call_spotify_and_dispatch_many(move || async move {
                api.save_tracks(ids).await?;
                Ok(vec![
                    AppAction::SaveSelection,
                    AppAction::ShowNotification(gettext("Tracks saved!")),
                ])
            })
    }

    fn remove_saved_tracks(&self) {
        let api = self.app_model.get_spotify();
        let ids: Vec<String> = self
            .selection()
            .peek_selection()
            .map(|s| &s.id)
            .cloned()
            .collect();
        self.dispatcher
            .call_spotify_and_dispatch_many(move || async move {
                api.remove_saved_tracks(ids).await?;
                Ok(vec![AppAction::UnsaveSelection])
            })
    }

    fn selection(&self) -> impl Deref<Target = SelectionState> + '_ {
        self.app_model.map_state(|s| &s.selection)
    }

    fn selected_count(&self) -> usize {
        self.selection().count()
    }

    fn user_playlists(&self) -> impl Deref<Target = Vec<PlaylistSummary>> + '_ {
        self.app_model.map_state(|s| &s.logged_user.playlists)
    }

    fn add_to_playlist(&self, id: &str) {
        let id = id.to_string();
        let api = self.app_model.get_spotify();
        let uris: Vec<String> = self
            .selection()
            .peek_selection()
            .map(|s| &s.uri)
            .cloned()
            .collect();
        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.add_to_playlist(&id, uris).await?;
                Ok(SelectionAction::Clear.into())
            })
    }

    fn remove_from_playlist(&self, id: &str) {
        let api = self.app_model.get_spotify();
        let id = id.to_string();
        let uris: Vec<String> = self
            .selection()
            .peek_selection()
            .map(|s| &s.uri)
            .cloned()
            .collect();
        self.dispatcher
            .call_spotify_and_dispatch_many(move || async move {
                api.remove_from_playlist(&id, uris.clone()).await?;
                Ok(vec![
                    BrowserAction::RemoveTracksFromPlaylist(id, uris).into(),
                    SelectionAction::Clear.into(),
                ])
            })
    }
}

pub struct SelectionToolbar {
    model: Rc<SelectionToolbarModel>,
    widget: SelectionToolbarWidget,
}

impl SelectionToolbar {
    pub fn new(model: SelectionToolbarModel, widget: SelectionToolbarWidget) -> Self {
        let model = Rc::new(model);
        widget.connect_move_up(clone!(@weak model => move || model.move_up_selection()));
        widget.connect_move_down(clone!(@weak model => move || model.move_down_selection()));
        widget.connect_queue(clone!(@weak model => move || model.queue_selection()));
        widget.connect_remove(clone!(@weak model => move || model.remove_selection()));
        widget.connect_save(clone!(@weak model => move || model.save_selection()));
        Self { model, widget }
    }

    fn update_active_tools(&self) {
        let count = self.model.selected_count();
        match self.model.selection().context {
            SelectionContext::Default => {
                self.widget.set_move(SelectionToolState::Hidden);
                self.widget
                    .set_queue(SelectionToolState::Visible(count > 0));
                self.widget.set_add(SelectionToolState::Visible(count > 0));
                self.widget.set_remove(SelectionToolState::Hidden);
                self.widget.set_save(SelectionToolState::Visible(count > 0));
            }
            SelectionContext::SavedTracks => {
                self.widget.set_move(SelectionToolState::Hidden);
                self.widget
                    .set_queue(SelectionToolState::Visible(count > 0));
                self.widget.set_add(SelectionToolState::Visible(count > 0));
                self.widget
                    .set_remove(SelectionToolState::Visible(count > 0));
                self.widget.set_save(SelectionToolState::Hidden);
            }
            SelectionContext::ReadOnlyQueue => {
                self.widget.set_move(SelectionToolState::Hidden);
                self.widget.set_queue(SelectionToolState::Hidden);
                self.widget.set_add(SelectionToolState::Hidden);
                self.widget.set_remove(SelectionToolState::Hidden);
                self.widget.set_save(SelectionToolState::Visible(count > 0));
            }
            SelectionContext::Queue => {
                self.widget
                    .set_move(SelectionToolState::Visible(count == 1));
                self.widget.set_queue(SelectionToolState::Hidden);
                self.widget.set_add(SelectionToolState::Hidden);
                self.widget
                    .set_remove(SelectionToolState::Visible(count > 0));
                self.widget.set_save(SelectionToolState::Visible(count > 0));
            }
            SelectionContext::Playlist => {
                self.widget.set_move(SelectionToolState::Hidden);
                self.widget
                    .set_queue(SelectionToolState::Visible(count > 0));
                self.widget.set_add(SelectionToolState::Hidden);
                self.widget.set_remove(SelectionToolState::Hidden);
                self.widget.set_save(SelectionToolState::Hidden);
            }
            SelectionContext::EditablePlaylist(_) => {
                self.widget.set_move(SelectionToolState::Hidden);
                self.widget
                    .set_queue(SelectionToolState::Visible(count > 0));
                self.widget.set_add(SelectionToolState::Hidden);
                self.widget
                    .set_remove(SelectionToolState::Visible(count > 0));
                self.widget.set_save(SelectionToolState::Hidden);
            }
        };
    }
}

impl Component for SelectionToolbar {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.upcast_ref()
    }
}

impl EventListener for SelectionToolbar {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(active)) => {
                self.widget.set_visible(*active);
                self.update_active_tools();
            }
            AppEvent::SelectionEvent(SelectionEvent::SelectionChanged) => {
                self.update_active_tools();
            }
            AppEvent::LoginEvent(LoginEvent::UserPlaylistsLoaded) => {
                let model = &self.model;
                self.widget.connect_playlists(
                    &model.user_playlists(),
                    clone!(@weak model => move |s| model.add_to_playlist(s)),
                );
            }
            _ => {}
        }
    }
}
