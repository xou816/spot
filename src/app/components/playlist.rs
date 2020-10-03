use gtk::prelude::*;
use gtk::{ListBoxExt};
use gio::prelude::*;
use gio::ListModelExt;

use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};

use crate::app::{AppAction, SongDescription};
use crate::app::components::{Component};

use super::gtypes::SongModel;

pub trait PlaylistModel {
    fn songs(&self) -> Ref<'_, Vec<SongDescription>>;
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

        for (i, song) in self.model.songs().iter().enumerate() {

            let is_current = current_song_uri.clone().map(|uri| *uri == song.uri);

            if let (Some(is_current), Some(model_song)) = (is_current, self.model_song_at(i)) {
                model_song.set_name(&song_name_for(song, is_current)[..]);
            }
        }
    }

    fn reset_list(&self) {
        let list_model = &self.list_model;

        list_model.remove_all();
        for song in self.model.songs().iter() {
            list_model.append(&SongModel::new(&song_name_for(song, false)[..], &song.uri));
        }
    }

}

impl Component for Playlist {
    fn handle(&self, action: &AppAction) {
        match action {
            AppAction::Load(_) => {
                self.update_list();
            },
            AppAction::LoadPlaylist(_) => {
                self.reset_list()
            }
            _ => {}
        }
    }
}

impl Playlist {

    fn create_button_for(song_uri: String, model: Weak<dyn PlaylistModel>) -> gtk::Button {
        let button = gtk::Button::new();
        let image = gtk::Image::new_from_icon_name(Some("media-playback-start"), gtk::IconSize::Button);
        button.add(&image);
        button.set_relief(gtk::ReliefStyle::None);

        button.connect_clicked(move |_| {
            model.upgrade().map(|m| m.play_song(song_uri.clone()));
        });

        button
    }

    fn create_row_for(item: &SongModel, model: Weak<dyn PlaylistModel>) -> gtk::ListBoxRow {
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

            let button = Playlist::create_button_for(item_uri, model);
            hbox.pack_start(&button, false, false, 0);
        }

        row.add(&hbox);
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
