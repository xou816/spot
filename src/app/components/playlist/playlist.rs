use gio::prelude::*;
use gtk::prelude::*;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::utils::AnimatorDefault;
use crate::app::components::{Component, EventListener, SongWidget};
use crate::app::models::SongModel;
use crate::app::{
    state::{PlaybackEvent, SelectionEvent, SelectionState},
    AppEvent, ListDiff, ListStore, Worker,
};

#[derive(Clone, Copy, Debug)]
struct RowState {
    is_selected: bool,
    is_playing: bool,
}

pub trait PlaylistModel {
    fn current_song_id(&self) -> Option<String>;

    fn play_song_at(&self, pos: usize, id: &str);

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
    animator: AnimatorDefault,
    listview: gtk::ListView,
    _press_gesture: gtk::GestureLongPress,
    list_model: ListStore<SongModel>,
    model: Rc<Model>,
}

impl<Model> Playlist<Model>
where
    Model: PlaylistModel + 'static,
{
    pub fn new(listview: gtk::ListView, model: Rc<Model>, worker: Worker) -> Self {
        let list_model = ListStore::new();
        let selection_model = gtk::NoSelection::new(Some(list_model.unsafe_store()));
        let factory = gtk::SignalListItemFactory::new();

        listview.set_factory(Some(&factory));
        listview.style_context().add_class("playlist");
        listview.set_single_click_activate(true);
        listview.set_model(Some(&selection_model));
        Self::set_selection_active(&listview, model.is_selection_enabled());

        factory.connect_setup(|_, item| {
            item.set_child(Some(&SongWidget::new()));
        });

        factory.connect_bind(clone!(@weak model => move |_, item| {
            let song_model = item.item().unwrap().downcast::<SongModel>().unwrap();
            song_model.set_state(Self::get_item_state(&*model, &song_model));

            let widget = item.child().unwrap().downcast::<SongWidget>().unwrap();
            widget.bind(&song_model, worker.clone());

            let id = &song_model.get_id();
            widget.set_actions(model.actions_for(id).as_ref());
            widget.set_menu(model.menu_for(id).as_ref());
        }));

        factory.connect_unbind(|_, item| {
            let song_model = item.item().unwrap().downcast::<SongModel>().unwrap();
            song_model.unbind_all();
        });

        listview.connect_activate(clone!(@weak list_model, @weak model => move |_, position| {
            let song = list_model.get(position);
            let selection_enabled = model.is_selection_enabled();
            if selection_enabled {
                Self::select_song(&*model, &song);
            } else {
                model.play_song_at(position as usize, &song.get_id());
            }
        }));

        let press_gesture = gtk::GestureLongPress::new();
        listview.add_controller(&press_gesture);
        press_gesture.set_touch_only(false);
        press_gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
        press_gesture.connect_pressed(clone!(@weak model => move |_, _, _| {
            model.enable_selection();
        }));

        Self {
            animator: AnimatorDefault::ease_in_out_animator(),
            listview,
            _press_gesture: press_gesture,
            list_model,
            model,
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

    fn get_item_state(model: &Model, item: &SongModel) -> RowState {
        let id = &item.get_id();
        let is_playing = model.current_song_id().map(|s| s.eq(id)).unwrap_or(false);
        let is_selected = model
            .selection()
            .map(|s| s.is_song_selected(id))
            .unwrap_or(false);
        RowState {
            is_selected,
            is_playing,
        }
    }

    fn autoscroll_to_playing(&self, index: usize) {
        let len = self.list_model.len() as f64;
        let adj = self
            .listview
            .parent()
            .and_then(|p| p.downcast::<gtk::ScrolledWindow>().ok())
            .and_then(|w| w.vadjustment());
        if let Some(adj) = adj {
            let v = adj.value();
            let pos = (index as f64) * adj.upper() / len;
            if pos < v || pos > v + 0.9 * adj.page_size() {
                self.animator.animate(
                    20,
                    clone!(@weak adj => @default-return false, move |p| {
                        let v = adj.value();
                        adj.set_value(v + p * (pos - v));
                        true
                    }),
                );
            }
        }
    }

    fn update_list(&self) {
        for (i, model_song) in self.list_model.iter().enumerate() {
            let state = Self::get_item_state(&*self.model, &model_song);
            model_song.set_state(state);
            if state.is_playing
                && self.model.autoscroll_to_playing()
                && !self.model.is_selection_enabled()
            {
                self.autoscroll_to_playing(i);
            }
        }
    }

    fn set_selection_active(listview: &gtk::ListView, active: bool) {
        let context = listview.style_context();
        if active {
            context.add_class("playlist--selectable");
        } else {
            context.remove_class("playlist--selectable");
        }
    }
}

impl SongModel {
    fn set_state(&self, state: RowState) {
        self.set_playing(state.is_playing);
        self.set_selected(state.is_selected);
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
                    self.update_list();
                }
                AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) => {
                    self.update_list();
                }
                AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(_)) => {
                    Self::set_selection_active(&self.listview, self.model.is_selection_enabled());
                    self.update_list();
                }
                _ => {}
            }
        }
    }
}

impl<Model> Component for Playlist<Model> {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.listview.upcast_ref()
    }
}
