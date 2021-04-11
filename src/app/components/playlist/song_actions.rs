use gdk::SELECTION_CLIPBOARD;
use gio::SimpleAction;
use gtk::Clipboard;

use crate::app::models::SongDescription;
use crate::app::state::{AppAction, PlaybackAction};
use crate::app::ActionDispatcher;

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
            let clipboard = Clipboard::get(&SELECTION_CLIPBOARD);
            clipboard.set_text(&format!("https://open.spotify.com/track/{}", &track_id));
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
}
