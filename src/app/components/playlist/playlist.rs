use gio::prelude::*;
use gtk::prelude::*;
use gtk::ListBoxExt;
use std::rc::{Rc, Weak};

use crate::app::components::{Component, EventListener, Song};
use crate::app::models::SongModel;
use crate::app::{state::PlaybackEvent, AppEvent, ListStore};

pub trait PlaylistModel {
    fn songs(&self) -> Vec<SongModel>;
    fn current_song_id(&self) -> Option<String>;
    fn play_song(&self, id: &str);
    fn select_song(&self, id: &str);
    fn is_song_selected(&self, id: &str) -> bool;
    fn is_selection_enabled(&self) -> bool;
    fn should_refresh_songs(&self, event: &AppEvent) -> bool;
    fn actions_for(&self, id: &str) -> Option<gio::ActionGroup>;
    fn menu_for(&self, id: &str) -> Option<gio::MenuModel>;
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

        listbox.set_selection_mode(gtk::SelectionMode::Multiple);
        listbox.get_style_context().add_class("playlist");
        listbox.set_activate_on_single_click(true);

        let list_model_clone = list_model.clone();
        listbox.connect_row_activated(clone!(@weak model => move |listbox, row| {
            if model.is_selection_enabled() {
                row.set_selectable(true);
                if row.is_selected() {
                    listbox.unselect_row(row);
                    row.set_selectable(false);
                } else {
                    listbox.select_row(Some(row));
                }
            } else {
                let index = row.get_index() as u32;
                let song: SongModel = list_model_clone.get(index);
                model.play_song(&song.get_id());
            }
        }));

        let weak_model = Rc::downgrade(&model);
        listbox.bind_model(Some(list_model.unsafe_store()), move |item| {
            let item = item.downcast_ref::<SongModel>().unwrap();
            let row = Self::create_row_for(
                item,
                weak_model.clone(),
                weak_model
                    .upgrade()
                    .and_then(|m| m.actions_for(&item.get_id())),
                weak_model
                    .upgrade()
                    .and_then(|m| m.menu_for(&item.get_id())),
            );
            row.set_selectable(false);
            row.show_all();
            row.upcast::<gtk::Widget>()
        });

        Self { list_model, model }
    }

    fn create_row_for<M: PlaylistModel>(
        item: &SongModel,
        model: Weak<M>,
        actions: Option<gio::ActionGroup>,
        menu: Option<gio::MenuModel>,
    ) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();

        let is_current = model
            .upgrade()
            .and_then(|model| {
                let current_song_id = model.current_song_id();
                current_song_id.as_ref().map(|s| s.eq(&item.get_id()))
            })
            .unwrap_or(false);

        item.set_playing(is_current);

        let song = Song::new(item.clone());
        row.add(song.get_root_widget());
        song.set_menu(menu.as_ref());
        song.set_actions(actions.as_ref());
        row
    }

    fn update_list(&self) {
        for (i, song) in self.model.songs().iter().enumerate() {
            let is_current = self
                .model
                .current_song_id()
                .map(|s| &s[..] == song.get_id())
                .unwrap_or(false);
            let model_song = self.list_model.get(i as u32);
            model_song.set_playing(is_current);
        }
    }

    fn reset_list(&mut self) {
        let list_model = &mut self.list_model;
        list_model.replace_all(self.model.songs());
    }
}

impl<Model> EventListener for Playlist<Model>
where
    Model: PlaylistModel + 'static,
{
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) => {
                self.update_list();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackStopped) => {
                self.reset_list();
            }
            _ if self.model.should_refresh_songs(event) => self.reset_list(),
            _ => {}
        }
    }
}
