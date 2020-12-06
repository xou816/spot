use gtk::prelude::*;
use gtk::ListBoxExt;
use gio::prelude::*;
use gio::ListModelExt;

use std::rc::{Rc, Weak};
use std::cell::Ref;

use crate::app::{AppEvent, BrowserEvent, SongDescription};
use crate::app::components::{Component, EventListener, Song};
use crate::app::components::gtypes::SongModel;

pub trait PlaylistModel {
    fn songs(&self) -> Option<Ref<'_, Vec<SongDescription>>>;
    fn current_song_uri(&self) -> Option<String>;
    fn play_song(&self, uri: String);
}

pub struct Playlist {
    list_model: gio::ListStore,
    model: Rc<dyn PlaylistModel>
}

impl Playlist {

    pub fn new(listbox: gtk::ListBox, model: Rc<dyn PlaylistModel>) -> Self {

        let list_model = gio::ListStore::new(SongModel::static_type());
        let weak_model = Rc::downgrade(&model);

        listbox.set_selection_mode(gtk::SelectionMode::None);
        listbox.get_style_context().add_class("playlist");

        listbox.bind_model(Some(&list_model), move |item| {
            let item = item.downcast_ref::<SongModel>().unwrap();
            let row = Playlist::create_row_for(&item, weak_model.clone());
            row.show_all();
            row.upcast::<gtk::Widget>()
        });


        Self { list_model, model }
    }

    fn model_song_at(&self, index: usize) -> Option<SongModel> {
        self.list_model.get_object(index as u32).and_then(|object| {
            object.downcast::<SongModel>().ok()
        })
    }

    fn update_list(&self) {
        let current_song_uri = self.model.current_song_uri();
        let current_song_uri = &current_song_uri;

        if let Some(songs) = self.model.songs() {
            for (i, song) in songs.iter().enumerate() {
                let is_current = current_song_uri.as_ref().map(|s| s.eq(&song.uri)).unwrap_or(false);
                if let Some(model_song) = self.model_song_at(i) {
                    model_song.set_playing(is_current);
                }
            }
        }

    }

    fn reset_list(&self) {

        let current_song_uri = self.model.current_song_uri();
        let current_song_uri = &current_song_uri;

        let list_model = &self.list_model;
        list_model.remove_all();

        if let Some(songs) = self.model.songs() {
            for song in songs.iter() {
                let is_current = current_song_uri.as_ref().map(|s| s.eq(&song.uri)).unwrap_or(false);
                list_model.append(&SongModel::new(&song.title, &song.artist, &song.uri));
            }
        }
    }

}

impl EventListener for Playlist {
    fn on_event(&self, event: &AppEvent) {
        match event {
            AppEvent::TrackChanged(_)|AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.update_list();
            },
            AppEvent::PlaylistChanged|AppEvent::BrowserEvent(BrowserEvent::DetailsLoaded) => {
                self.reset_list()
            }
            _ => {}
        }
    }
}

fn play_button_style(button: gtk::ButtonBuilder) -> gtk::ButtonBuilder {

    let image = gtk::Image::from_icon_name(
        Some("media-playback-start"),
        gtk::IconSize::Button);

    button
        .image(&image)
        .relief(gtk::ReliefStyle::None)
}

impl Playlist {

    fn create_row_for(item: &SongModel, model: Weak<dyn PlaylistModel>) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();

        let song = Song::new(item.clone());
        song.connect_play_pressed(move |song| {
            model.upgrade().map(|m| m.play_song(song.get_uri()));
        });

        row.add(song.get_root_widget());
        row
    }
}

fn song_name_for(song: &SongDescription, is_playing: bool) -> String {
    let title = glib::markup_escape_text(&song.title);
    let artist = glib::markup_escape_text(&song.artist);
    if is_playing {
        format!("<b>{} — <small>{}</small></b>", title.as_str(), artist.as_str())
    } else {
        format!("{} — <small>{}</small>", title.as_str(), artist.as_str())
    }
}
