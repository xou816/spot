use gio::prelude::*;
use gtk::prelude::*;
use gtk::ListBoxExt;

use std::cell::Ref;
use std::rc::{Rc, Weak};

use crate::app::components::{Component, EventListener, Song};
use crate::app::models::SongModel;
use crate::app::{AppEvent, BrowserEvent, ListStore, SongDescription};

pub trait PlaylistModel {
    fn songs(&self) -> Option<Ref<'_, Vec<SongDescription>>>;
    fn current_song_uri(&self) -> Option<String>;
    fn play_song(&self, uri: String);
}

pub struct Playlist {
    list_model: ListStore<SongModel>,
    model: Rc<dyn PlaylistModel>,
}

impl Playlist {
    pub fn new(listbox: gtk::ListBox, model: Rc<dyn PlaylistModel>) -> Self {
        let list_model = ListStore::new();
        let weak_model = Rc::downgrade(&model);

        listbox.set_selection_mode(gtk::SelectionMode::None);
        listbox.get_style_context().add_class("playlist");

        listbox.bind_model(Some(list_model.unsafe_store()), move |item| {
            let item = item.downcast_ref::<SongModel>().unwrap();
            let row = Self::create_row_for(item, weak_model.clone());
            row.show_all();
            row.upcast::<gtk::Widget>()
        });

        Self { list_model, model }
    }

    fn create_row_for(item: &SongModel, model: Weak<dyn PlaylistModel>) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();

        let is_current = model
            .upgrade()
            .and_then(|model| {
                let current_song_uri = model.current_song_uri();
                current_song_uri.as_ref().map(|s| s.eq(&item.get_uri()))
            })
            .unwrap_or(false);

        item.set_playing(is_current);

        let song = Song::new(item.clone());
        song.connect_play_pressed(move |song| {
            if let Some(m) = model.upgrade() {
                m.play_song(song.get_uri());
            }
        });

        row.add(song.get_root_widget());
        row
    }

    fn song_is_current(&self, song: &SongDescription) -> bool {
        let current_song_uri = self.model.current_song_uri();
        let current_song_uri = current_song_uri.as_ref();

        current_song_uri.map(|s| s.eq(&song.uri)).unwrap_or(false)
    }

    fn update_list(&self) {
        if let Some(songs) = self.model.songs() {
            for (i, song) in songs.iter().enumerate() {
                let is_current = self.song_is_current(song);
                let model_song = self.list_model.get(i as u32);
                model_song.set_playing(is_current);
            }
        }
    }

    fn reset_list(&mut self) {
        let list_model = &mut self.list_model;
        list_model.remove_all();

        if let Some(songs) = self.model.songs() {
            for (i, song) in songs.iter().enumerate() {
                let index = i as u32 + 1;
                list_model.append(SongModel::new(index, &song.title, &song.artist, &song.uri));
            }
        }
    }
}

impl EventListener for Playlist {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::TrackChanged(_) | AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.update_list();
            }
            AppEvent::PlaylistChanged | AppEvent::BrowserEvent(BrowserEvent::DetailsLoaded) => {
                self.reset_list()
            }
            _ => {}
        }
    }
}
