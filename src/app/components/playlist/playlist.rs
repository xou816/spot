use gio::prelude::*;
use gtk::prelude::*;
use gtk::ListBoxExt;

use std::cell::Ref;
use std::rc::{Rc, Weak};

use crate::app::components::{Component, EventListener, Song};
use crate::app::models::SongModel;
use crate::app::{AppEvent, ListStore, SongDescription};

pub trait PlaylistModel {
    fn songs(&self) -> Option<Ref<'_, Vec<SongDescription>>>;
    fn current_song_id(&self) -> Option<String>;
    fn play_song(&self, id: String);
    fn should_refresh_songs(&self, event: &AppEvent) -> bool;
    fn actions_for(&self, id: String) -> Option<gio::ActionGroup>;
    fn menu_for(&self, id: String) -> Option<gio::MenuModel>;
}

pub struct Playlist<Model> {
    list_model: ListStore<SongModel>,
    model: Rc<Model>,
}

impl<Model> Playlist<Model>
where
    Model: PlaylistModel + 'static,
{
    pub fn new(listbox: gtk::ListBox, model: Rc<Model>) -> Self {
        let list_model = ListStore::new();

        listbox.set_selection_mode(gtk::SelectionMode::None);
        listbox.get_style_context().add_class("playlist");
        listbox.set_activate_on_single_click(true);

        let list_model_clone = list_model.clone();
        let weak_model = Rc::downgrade(&model);
        listbox.connect_row_activated(move |_, row| {
            let index = row.get_index() as u32;
            let song: SongModel = list_model_clone.get(index);
            if let Some(m) = weak_model.upgrade() {
                m.play_song(song.get_uri());
            }
        });

        let weak_model = Rc::downgrade(&model);
        listbox.bind_model(Some(list_model.unsafe_store()), move |item| {
            let item = item.downcast_ref::<SongModel>().unwrap();
            let row = Self::create_row_for(
                item,
                weak_model.clone(),
                weak_model
                    .upgrade()
                    .and_then(|m| m.actions_for(item.get_uri())),
                weak_model
                    .upgrade()
                    .and_then(|m| m.menu_for(item.get_uri())),
            );
            row.show_all();
            row.upcast::<gtk::Widget>()
        });

        Self { list_model, model }
    }

    fn create_row_for(
        item: &SongModel,
        model: Weak<dyn PlaylistModel>,
        actions: Option<gio::ActionGroup>,
        menu: Option<gio::MenuModel>,
    ) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();

        let is_current = model
            .upgrade()
            .and_then(|model| {
                let current_song_id = model.current_song_id();
                current_song_id.as_ref().map(|s| s.eq(&item.get_uri()))
            })
            .unwrap_or(false);

        item.set_playing(is_current);

        let song = Song::new(item.clone());
        row.add(song.get_root_widget());
        song.set_menu(menu.as_ref());
        song.set_actions(actions.as_ref());
        row
    }

    fn song_is_current(&self, song: &SongDescription) -> bool {
        let current_song_id = self.model.current_song_id();
        let current_song_id = current_song_id.as_ref();

        current_song_id.map(|s| s.eq(&song.id)).unwrap_or(false)
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
                list_model.append(SongModel::new(
                    index,
                    &song.title,
                    &song.artists_name(),
                    &song.id,
                ));
            }
        }
    }
}

impl<Model> EventListener for Playlist<Model>
where
    Model: PlaylistModel + 'static,
{
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::TrackChanged(_) => {
                self.update_list();
            }
            _ if self.model.should_refresh_songs(event) => self.reset_list(),
            _ => {}
        }
    }
}
