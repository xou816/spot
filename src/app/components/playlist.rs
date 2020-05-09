use gtk::prelude::*;
use gtk::{ListBoxExt};
use gio::prelude::*;
use gio::ListModelExt;

use std::rc::Rc;
use std::cell::RefCell;

use crate::app::{AppAction, SongDescription};
use crate::app::components::{Component, Dispatcher};

use super::gtypes::Song;

pub trait PlaylistState {
    fn songs(&self) -> &Vec<SongDescription>;
    fn current_song_uri(&self) -> Option<String>;
}

pub struct Playlist<State: 'static> where State: PlaylistState {
    model: gio::ListStore,
    state: Rc<RefCell<State>>
}

impl<State> Playlist<State> where State: PlaylistState {

    pub fn new(builder: &gtk::Builder, state: Rc<RefCell<State>>, dispatcher: Dispatcher) -> Self {

        let listbox: gtk::ListBox = builder.get_object("listbox").unwrap();
        let model = gio::ListStore::new(Song::static_type());

        for song in state.borrow().songs().iter() {
            model.append(&Song::new(&song_name_for(song, false)[..], &song.uri));
        }

        let clone = dispatcher.clone();

        listbox.bind_model(Some(&model), move |item| {
            let item = item.downcast_ref::<Song>().unwrap();
            let row = create_row_for(&item, clone.clone());
            row.show_all();
            row.upcast::<gtk::Widget>()
        });

        Self { model, state }
    }

    fn model_song_at(&self, index: usize) -> Option<Song> {
        self.model.get_object(index as u32).and_then(|object| {
            object.downcast::<Song>().ok()
        })
    }

    fn update_list(&self) {
        let state = self.state.borrow();
        let current_song_uri = state.current_song_uri();

        for (i, song) in state.songs().iter().enumerate() {

            let is_current = current_song_uri.clone().map(|uri| *uri == song.uri);

            if let (Some(is_current), Some(model_song)) = (is_current, self.model_song_at(i)) {
                model_song.set_name(&song_name_for(song, is_current)[..]);
            }
        }
    }
}

impl<State> Component for Playlist<State> where State: PlaylistState {
    fn handle(&self, action: AppAction) {

        match action {
            AppAction::Load(_) => {
                self.update_list();
            },
            _ => {}
        }
    }
}

fn create_row_for(item: &Song, dispatcher: Dispatcher) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);

    if let Some(item_uri) = item.uri() {

        let label = gtk::Label::new(None);
        label.set_use_markup(true);
        label.set_halign(gtk::Align::Start);

        item.bind_property("name", &label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        hbox.pack_start(&label, true, true, 0);

        let button = gtk::Button::new();
        let image = gtk::Image::new_from_icon_name(Some("media-playback-start"), gtk::IconSize::Button);
        button.add(&image);
        button.set_relief(gtk::ReliefStyle::None);

        hbox.pack_start(&button, false, false, 0);

        button.connect_clicked(move |_| {
            dispatcher.send(AppAction::Load(item_uri.clone())).expect("Could not send");
        });
    }

    row.add(&hbox);
    row
}

fn song_name_for(song: &SongDescription, is_playing: bool) -> String {
    if is_playing {
        format!("<b>{} - {}</b>", song.title, song.artist)
    } else {
        format!("{} - {}", song.title, song.artist)
    }
}
