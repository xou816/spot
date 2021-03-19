use gio::prelude::*;
use gtk::prelude::*;
use gtk::ListBoxExt;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::{Component, EventListener, Song};
use crate::app::models::SongModel;
use crate::app::{
    state::{PlaybackEvent, SelectionEvent, SelectionState},
    AppEvent, ListStore,
};

pub trait PlaylistModel {
    fn songs(&self) -> Vec<SongModel>;
    fn current_song_id(&self) -> Option<String>;
    fn play_song(&self, id: &str);
    fn should_refresh_songs(&self, event: &AppEvent) -> bool;

    fn actions_for(&self, _id: &str) -> Option<gio::ActionGroup> {
        None
    }
    fn menu_for(&self, _id: &str) -> Option<gio::MenuModel> {
        None
    }

    fn select_song(&self, _id: &str) {}
    fn deselect_song(&self, _id: &str) {}

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        None
    }
}

pub struct Playlist<Model> {
    listbox: gtk::ListBox,
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
        listbox.connect_row_activated(clone!(@weak model => move |listbox, row| {
            let index = row.get_index() as u32;
            let song: SongModel = list_model_clone.get(index);
            let selection_enabled = model.selection().map(|s| s.is_selection_enabled()).unwrap_or(false);
            if selection_enabled {
                Self::select_song(&listbox, &row, &song, &*model);
            } else {
                model.play_song(&song.get_id());
            }
        }));

        listbox.bind_model(
            Some(list_model.unsafe_store()),
            clone!(@weak model, @weak listbox => @default-panic, move |item| {
                let item = item.downcast_ref::<SongModel>().unwrap();
                let id = &item.get_id();

                let row = gtk::ListBoxRow::new();
                let event_box = gtk::EventBox::new();
                row.add(&event_box);

                let song = Song::new(item.clone());
                event_box.add(song.get_root_widget());

                song.set_menu(model.menu_for(id).as_ref());
                song.set_actions(model.actions_for(id).as_ref());

                Self::set_row_state(&listbox, item, &row, &*model);
                Self::connect_events(&listbox, item, &row, model);

                row.show_all();
                row.upcast::<gtk::Widget>()
            }),
        );

        Self {
            listbox,
            list_model,
            model,
        }
    }

    fn select_song(listbox: &gtk::ListBox, row: &gtk::ListBoxRow, song: &SongModel, model: &Model) {
        row.set_selectable(true);
        if row.is_selected() {
            listbox.unselect_row(row);
            row.set_selectable(false);
            model.deselect_song(&song.get_id());
        } else {
            listbox.select_row(Some(row));
            model.select_song(&song.get_id());
        }
    }

    fn connect_events(
        listbox: &gtk::ListBox,
        item: &SongModel,
        row: &gtk::ListBoxRow,
        model: Rc<Model>,
    ) {
        row.connect_button_release_event(
            clone!(@weak model, @weak listbox, @strong item => @default-return Inhibit(false), move |row, event| {
                if event.get_button() == 3 {
                    listbox.set_selection_mode(gtk::SelectionMode::Multiple);
                    Self::select_song(&listbox, row, &item, &*model);
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            }),
        );
    }

    fn set_row_state(
        listbox: &gtk::ListBox,
        item: &SongModel,
        row: &gtk::ListBoxRow,
        model: &Model,
    ) {
        let id = &item.get_id();
        let current_song_id = model.current_song_id();
        let is_current = current_song_id.as_ref().map(|s| s.eq(id)).unwrap_or(false);
        let is_selected = model
            .selection()
            .map(|s| s.is_song_selected(id))
            .unwrap_or(false);

        item.set_playing(is_current);
        if is_selected {
            row.set_selectable(true);
            listbox.select_row(Some(row));
        } else {
            row.set_selectable(false);
        }
    }

    fn update_list(&self) {
        for (i, song) in self.model.songs().iter().enumerate() {
            let is_current = self
                .model
                .current_song_id()
                .map(|s| s == song.get_id())
                .unwrap_or(false);
            let model_song = self.list_model.get(i as u32);
            model_song.set_playing(is_current);
        }
    }

    fn reset_list(&mut self) {
        let list_model = &mut self.list_model;
        list_model.replace_all(self.model.songs());
    }

    fn set_selection_active(&self, active: bool) {
        if active {
            self.listbox
                .set_selection_mode(gtk::SelectionMode::Multiple);
        } else {
            for row in self.listbox.get_selected_rows() {
                self.listbox.unselect_row(&row);
                row.set_selectable(false);
            }
            self.listbox.set_selection_mode(gtk::SelectionMode::None);
        }
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
            AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(active)) => {
                self.set_selection_active(*active);
            }
            _ if self.model.should_refresh_songs(event) => self.reset_list(),
            _ => {}
        }
    }
}
