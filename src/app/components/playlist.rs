use gtk::prelude::*;
use gtk::ListBoxExt;
use gio::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

use crate::app::AppAction;
use crate::app::components::{Component, Dispatcher};

use super::gtypes::Song;

struct PlaylistState {
    songs: Vec<Song>,
    position: i32
}

pub struct Playlist {
    listbox: gtk::ListBox,
    state: Rc<RefCell<PlaylistState>>
}

impl Playlist {

    pub fn new(builder: &gtk::Builder, dispatcher: Dispatcher) -> Self {

        let listbox: gtk::ListBox = builder.get_object("listbox").unwrap();

        let songs = vec![
            Song::new("spotify:track:6j67aNAPeQ31uw4qw4rpLa"),
            Song::new("spotify:track:1swmf4hFMJYRNA8Rq9PVaW")
        ];

        let model = gio::ListStore::new(Song::static_type());
        model.append(songs.iter().next().unwrap());


        let state = Rc::new(RefCell::new(PlaylistState {
            songs, position: 0
        }));

        let clone = dispatcher.clone();

        listbox.bind_model(Some(&model), move |item| {
            let item = item.downcast_ref::<Song>().unwrap();
            let row = Playlist::create_row_for(&item, clone.clone());
            row.show_all();
            row.upcast::<gtk::Widget>()
        });

        Self { listbox, state }
    }

    fn create_row_for(item: &Song, dispatcher: Dispatcher) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();

        if let Some(item_name) = item.name() {
            let label = gtk::Label::new(Some(item_name.as_str()));

            row.add(&label);
            row.connect_activate(move |_| {
                dispatcher.send(AppAction::Load(item_name.clone())).expect("Could not send")
            });
        }

        row
    }
}

impl Component for Playlist {
    fn handle(&self, _: AppAction) {}
}
