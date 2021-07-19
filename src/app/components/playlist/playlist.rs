use gio::prelude::*;
use gtk::prelude::*;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::utils::{in_viewport, vscroll_to, AnimatorDefault};
use crate::app::components::{Component, EventListener, Song};
use crate::app::models::SongModel;
use crate::app::{
    state::{PlaybackEvent, SelectionEvent, SelectionState},
    AppEvent, ListDiff, ListStore,
};

#[derive(Clone, Copy, Debug)]
struct RowState {
    is_selected: bool,
    is_playing: bool,
}

pub trait PlaylistModel {
    fn current_song_id(&self) -> Option<String>;
    fn play_song(&self, id: &str);
    fn diff_for_event(&self, event: &AppEvent) -> Option<ListDiff<SongModel>>;

    fn autoscroll_to_playing(&self) -> bool {
        true
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
        listbox.style_context().add_class("playlist");
        listbox.set_activate_on_single_click(true);

        let press_gesture = gtk::GestureLongPress::new();
        listbox.add_controller(&press_gesture);
        press_gesture.set_touch_only(false);
        press_gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
        press_gesture.connect_pressed(clone!(@weak model => move |_, _, _| {
            model.enable_selection();
        }));

        let list_model_clone = list_model.clone();
        listbox.connect_row_activated(clone!(@weak model => move |_, row| {
            let index = row.index() as u32;
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
                let song = Song::new(item.clone());
                row.set_child(Some(song.get_root_widget()));

                song.set_menu(model.menu_for(id).as_ref());
                song.set_actions(model.actions_for(id).as_ref());

                Self::set_row_state(&listbox, item, &row, Self::get_row_state(item, &*model, None));

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
        item.set_selected(state.is_selected);
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
            .filter_map(move |(i, song)| listbox.row_at_index(i as i32).map(|r| (r, song)))
    }

    fn update_list(&self, scroll: bool) {
        let autoscroll = scroll && self.model.autoscroll_to_playing();
        let current_song_id = self.model.current_song_id();
        for (row, model_song) in self.rows_and_songs() {
            let state = Self::get_row_state(&model_song, &*self.model, current_song_id.as_ref());
            Self::set_row_state(&self.listbox, &model_song, &row, state);

            if state.is_playing && autoscroll && !in_viewport(row.upcast_ref()).unwrap_or(true) {
                self.animator
                    .animate(20, move |p| vscroll_to(row.upcast_ref(), p).is_some());
            }
        }
    }

    fn set_selection_active(listbox: &gtk::ListBox, active: bool) {
        let context = listbox.style_context();
        if active {
            context.add_class("playlist--selectable");
            listbox.set_selection_mode(gtk::SelectionMode::Multiple);
        } else {
            context.remove_class("playlist--selectable");
            listbox.set_selection_mode(gtk::SelectionMode::None);
        }
    }
}

impl<Model> EventListener for Playlist<Model>
where
    Model: PlaylistModel + 'static,
{
    fn on_event(&mut self, event: &AppEvent) {
        if let Some(diff) = self.model.diff_for_event(event) {
            self.list_model.update(diff);
        } else {
            match event {
                AppEvent::SelectionEvent(SelectionEvent::SelectionChanged) => {
                    self.update_list(false);
                }
                AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) => {
                    self.update_list(true);
                }
                AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(_)) => {
                    Self::set_selection_active(&self.listbox, self.model.is_selection_enabled());
                    self.update_list(false);
                }
                _ => {}
            }
        }
    }
}

impl<Model> Component for Playlist<Model> {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.listbox.upcast_ref()
    }
}
