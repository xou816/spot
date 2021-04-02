use gio::prelude::*;
use gtk::prelude::*;
use gtk::ListBoxExt;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::utils::{in_viewport, vscroll_to, AnimatorDefault};
use crate::app::components::{Component, EventListener, Song};
use crate::app::models::SongModel;
use crate::app::{
    state::{PlaybackEvent, SelectionEvent, SelectionState},
    AppEvent, ListStore,
};

#[derive(Clone, Copy, Debug)]
struct RowState {
    is_selected: bool,
    is_playing: bool,
}

pub trait PlaylistModel {
    fn songs(&self) -> Vec<SongModel>;
    fn current_song_id(&self) -> Option<String>;
    fn play_song(&self, id: &str);
    fn should_refresh_songs(&self, event: &AppEvent) -> bool;

    fn autoscroll_to_playing(&self) -> bool {
        false
    }

    fn actions_for(&self, _id: &str) -> Option<gio::ActionGroup> {
        None
    }
    fn menu_for(&self, _id: &str) -> Option<gio::MenuModel> {
        None
    }

    fn select_song(&self, _id: &str) {}
    fn deselect_song(&self, _id: &str) {}
    fn enable_selection(&self) -> bool {
        false
    }

    fn selection(&self) -> Option<Box<dyn Deref<Target = SelectionState> + '_>> {
        None
    }

    fn is_selection_enabled(&self) -> bool {
        self.selection()
            .map(|s| s.is_selection_enabled())
            .unwrap_or(false)
    }
}

pub struct Playlist<Model> {
    listbox: gtk::ListBox,
    _press_gesture: gtk::GestureLongPress,
    list_model: ListStore<SongModel>,
    model: Rc<Model>,
    animator: AnimatorDefault,
}

impl<Model> Playlist<Model>
where
    Model: PlaylistModel + 'static,
{
    pub fn new(listbox: gtk::ListBox, model: Rc<Model>) -> Self {
        let list_model = ListStore::new();

        Self::set_selection_active(&listbox, model.is_selection_enabled());
        listbox.get_style_context().add_class("playlist");
        listbox.set_activate_on_single_click(true);

        let press_gesture = gtk::GestureLongPress::new(&listbox);
        listbox.add_events(gdk::EventMask::TOUCH_MASK);
        press_gesture.set_touch_only(false);
        press_gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
        press_gesture.connect_pressed(clone!(@weak model => move |_, _, _| {
            model.enable_selection();
        }));

        let list_model_clone = list_model.clone();
        listbox.connect_row_activated(clone!(@weak model => move |_, row| {
            let index = row.get_index() as u32;
            let song: SongModel = list_model_clone.get(index);
            let selection_enabled = model.is_selection_enabled();
            if selection_enabled {
                Self::select_song(&*model, &song);
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

                Self::set_row_state(&listbox, item, &row, Self::get_row_state(item, &*model, None));
                Self::connect_events(item, &row, model);

                row.show_all();
                row.upcast::<gtk::Widget>()
            }),
        );

        Self {
            listbox,
            _press_gesture: press_gesture,
            list_model,
            model,
            animator: AnimatorDefault::ease_in_out_animator(),
        }
    }

    fn select_song(model: &Model, song: &SongModel) {
        if let Some(selection) = model.selection() {
            if selection.is_song_selected(&song.get_id()) {
                model.deselect_song(&song.get_id());
            } else {
                model.select_song(&song.get_id());
            }
        }
    }

    fn connect_events(item: &SongModel, row: &gtk::ListBoxRow, model: Rc<Model>) {
        row.connect_button_release_event(
            clone!(@weak model, @strong item => @default-return Inhibit(false), move |_, event| {
                if event.get_button() == 3 && model.enable_selection() {
                    Self::select_song(&*model, &item);
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            }),
        );
    }

    fn get_row_state(
        item: &SongModel,
        model: &Model,
        current_song_id: Option<&String>,
    ) -> RowState {
        let id = &item.get_id();
        let is_playing = current_song_id
            .map(|s| s.eq(id))
            .or_else(|| Some(model.current_song_id()?.eq(id)))
            .unwrap_or(false);
        let is_selected = model
            .selection()
            .map(|s| s.is_song_selected(id))
            .unwrap_or(false);
        RowState {
            is_selected,
            is_playing,
        }
    }

    fn set_row_state(
        listbox: &gtk::ListBox,
        item: &SongModel,
        row: &gtk::ListBoxRow,
        state: RowState,
    ) {
        item.set_playing(state.is_playing);
        if state.is_selected {
            row.set_selectable(true);
            listbox.select_row(Some(row));
        } else {
            row.set_selectable(false);
        }
    }

    fn rows_and_songs(&self) -> impl Iterator<Item = (gtk::ListBoxRow, SongModel)> + '_ {
        let listbox = &self.listbox;
        self.list_model
            .iter()
            .enumerate()
            .filter_map(move |(i, song)| listbox.get_row_at_index(i as i32).map(|r| (r, song)))
    }

    fn update_list(&self) {
        let autoscroll = self.model.autoscroll_to_playing();
        let current_song_id = self.model.current_song_id();
        for (row, model_song) in self.rows_and_songs() {
            let state = Self::get_row_state(&model_song, &*self.model, current_song_id.as_ref());
            Self::set_row_state(&self.listbox, &model_song, &row, state);

            if state.is_playing && autoscroll {
                if !in_viewport(row.upcast_ref()).unwrap_or(true) {
                    self.animator
                        .animate(20, move |p| vscroll_to(row.upcast_ref(), p).is_some());
                }
            }
        }
    }

    fn reset_list(&mut self) {
        let list_model = &mut self.list_model;
        list_model.replace_all(self.model.songs());
    }

    fn set_selection_active(listbox: &gtk::ListBox, active: bool) {
        if active {
            listbox.set_selection_mode(gtk::SelectionMode::Multiple);
        } else {
            for row in listbox.get_selected_rows() {
                listbox.unselect_row(&row);
                row.set_selectable(false);
            }
            listbox.set_selection_mode(gtk::SelectionMode::None);
        }
    }
}

impl<Model> EventListener for Playlist<Model>
where
    Model: PlaylistModel + 'static,
{
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_))
            | AppEvent::SelectionEvent(SelectionEvent::SelectionChanged) => {
                self.update_list();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackStopped) => {
                self.reset_list();
            }
            AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(active)) => {
                Self::set_selection_active(&self.listbox, *active);
            }
            _ if self.model.should_refresh_songs(event) => self.reset_list(),
            _ => {}
        }
    }
}

impl<Model> Component for Playlist<Model> {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.listbox.upcast_ref()
    }
}
