use gdk::prelude::*;
use gettextrs::gettext;
use gio::SimpleAction;
use std::rc::Rc;

use crate::app::models::SongDescription;
use crate::app::state::{AppAction, PlaybackAction, SelectionAction};
use crate::app::{ActionDispatcher, AppModel};

impl SongDescription {
    pub fn make_queue_action(
        &self,
        dispatcher: Box<dyn ActionDispatcher>,
        name: Option<&str>,
    ) -> SimpleAction {
        let queue = SimpleAction::new(name.unwrap_or("queue"), None);
        let song = self.clone();
        queue.connect_activate(move |_, _| {
            dispatcher.dispatch(PlaybackAction::Queue(vec![song.clone()]).into());
        });
        queue
    }

    pub fn make_dequeue_action(
        &self,
        dispatcher: Box<dyn ActionDispatcher>,
        name: Option<&str>,
    ) -> SimpleAction {
        let dequeue = SimpleAction::new(name.unwrap_or("dequeue"), None);
        let track_id = self.id.clone();
        dequeue.connect_activate(move |_, _| {
            dispatcher.dispatch(PlaybackAction::Dequeue(track_id.clone()).into());
        });
        dequeue
    }

    pub fn make_link_action(&self, name: Option<&str>) -> SimpleAction {
        let track_id = self.id.clone();
        let copy_link = SimpleAction::new(name.unwrap_or("copy_link"), None);
        copy_link.connect_activate(move |_, _| {
            let link = format!("https://open.spotify.com/track/{}", &track_id);
            let clipboard = gdk::Display::default().unwrap().clipboard();
            clipboard
                .set_content(Some(&gdk::ContentProvider::for_value(&link.to_value())))
                .expect("Failed to set clipboard content");
        });
        copy_link
    }

    pub fn make_album_action(
        &self,
        dispatcher: Box<dyn ActionDispatcher>,
        name: Option<&str>,
    ) -> SimpleAction {
        let album_id = self.album.id.clone();
        let view_album = SimpleAction::new(name.unwrap_or("view_album"), None);
        view_album.connect_activate(move |_, _| {
            dispatcher.dispatch(AppAction::ViewAlbum(album_id.clone()));
        });
        view_album
    }

    pub fn make_artist_actions(
        &self,
        dispatcher: Box<dyn ActionDispatcher>,
        prefix: Option<&str>,
    ) -> Vec<SimpleAction> {
        self.artists
            .iter()
            .map(|artist| {
                let id = artist.id.clone();
                let view_artist = SimpleAction::new(
                    &format!("{}_{}", prefix.unwrap_or("view_artist"), &id),
                    None,
                );
                let dispatcher = dispatcher.box_clone();
                view_artist.connect_activate(move |_, _| {
                    dispatcher.dispatch(AppAction::ViewArtist(id.clone()));
                });
                view_artist
            })
            .collect()
    }

    pub fn make_like_action(
        &self,
        dispatcher: Box<dyn ActionDispatcher>,
        app_model: Rc<AppModel>,
        name: Option<&str>,
    ) -> SimpleAction {
        let track_id = self.id.clone();
        let song = self.clone();
        let like_track = SimpleAction::new(name.unwrap_or("like"), None);
        like_track.connect_activate(move |_, _| {
            let track_id = track_id.clone();
            let song = song.clone();
            let api = app_model.get_spotify();
            dispatcher.dispatch(SelectionAction::Select(vec![song]).into());
            dispatcher.call_spotify_and_dispatch_many(move || async move {
                api.save_tracks(vec![track_id]).await?;
                Ok(vec![
                    AppAction::SaveSelection,
                    AppAction::ShowNotification(gettext("Track saved!")),
                ])
            });
        });
        like_track
    }

    pub fn make_unlike_action(
        &self,
        dispatcher: Box<dyn ActionDispatcher>,
        app_model: Rc<AppModel>,
        name: Option<&str>,
    ) -> SimpleAction {
        let track_id = self.id.clone();
        let song = self.clone();
        let unlike_track = SimpleAction::new(name.unwrap_or("unlike"), None);
        unlike_track.connect_activate(move |_, _| {
            let track_id = track_id.clone();
            let song = song.clone();
            let api = app_model.get_spotify();
            dispatcher.dispatch(SelectionAction::Select(vec![song]).into());
            dispatcher.call_spotify_and_dispatch_many(move || async move {
                api.remove_saved_tracks(vec![track_id]).await?;
                Ok(vec![
                    AppAction::UnsaveSelection,
                    AppAction::ShowNotification(gettext("Track unsaved!")),
                ])
            });
        });
        unlike_track
    }
}
