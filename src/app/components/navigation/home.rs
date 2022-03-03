use gettextrs::*;
use gtk::prelude::*;
use std::rc::Rc;

use super::HomeModel;
use crate::app::components::navigation::create_playlist::CreatePlaylistWidget;
use crate::app::components::sidebar_listbox::{SideBarItem, SideBarRow};
use crate::app::components::{Component, EventListener, SavedPlaylistsModel, ScreenFactory};
use crate::app::models::AlbumModel;
use crate::app::{AppEvent, BrowserEvent};

const LIBRARY: &str = "library";
const SAVED_TRACKS: &str = "saved_tracks";
const NOW_PLAYING: &str = "now_playing";
const SAVED_PLAYLISTS: &str = "saved_playlists";
const NEW_PLAYLIST: &str = "new_playlist";
const NUM_FIXED_ENTRIES: u32 = 6;
const NUM_PLAYLISTS: usize = 20;

fn add_to_stack_and_listbox(
    stack: &gtk::Stack,
    list_store: &gio::ListStore,
    widget: &gtk::Widget,
    name: &str,
    title: &str,
    icon_name: &str,
    grayed_out: bool,
) {
    stack.add_titled(widget, Option::from(name), title);
    list_store.append(&SideBarItem::new(name, title, icon_name, grayed_out))
}

fn make_playlist_item(playlist_item: AlbumModel) -> SideBarItem {
    let mut title = playlist_item.album_title();
    if title.is_empty() {
        title = gettext("Unnamed playlist");
    }

    let id = playlist_item.uri();

    SideBarItem::new(id.as_str(), &title, "playlist2-symbolic", false)
}

fn new_playlist_clicked(row: &gtk::ListBoxRow, model: Rc<HomeModel>) {
    CreatePlaylistWidget::new(row, model);
}

pub struct HomePane {
    model: Rc<HomeModel>,
    stack: gtk::Stack,
    listbox: gtk::ListBox,
    list_store: gio::ListStore,
    components: Vec<Box<dyn EventListener>>,
    saved_playlists_model: SavedPlaylistsModel,
}

impl HomePane {
    pub fn new(
        model: HomeModel,
        listbox: gtk::ListBox,
        screen_factory: &ScreenFactory,
        list_store: gio::ListStore,
    ) -> Self {
        let model = Rc::new(model);
        let library = screen_factory.make_library();
        let saved_playlists = screen_factory.make_saved_playlists();
        let saved_tracks = screen_factory.make_saved_tracks();
        let now_playing = screen_factory.make_now_playing();

        let saved_playlists_model = screen_factory.make_saved_playlists_model();
        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        add_to_stack_and_listbox(
            &stack,
            &list_store,
            library.get_root_widget(),
            LIBRARY,
            // translators: This is a sidebar entry to browse to saved albums.
            &gettext("Library"),
            "library-music-symbolic",
            false,
        );
        add_to_stack_and_listbox(
            &stack,
            &list_store,
            saved_tracks.get_root_widget(),
            SAVED_TRACKS,
            // translators: This is a sidebar entry to browse to saved tracks.
            &gettext("Saved tracks"),
            "starred-symbolic",
            false,
        );
        add_to_stack_and_listbox(
            &stack,
            &list_store,
            now_playing.get_root_widget(),
            NOW_PLAYING,
            &gettext("Now playing"),
            "music-queue-symbolic",
            false,
        );
        list_store.append(&SideBarItem::new(
            SAVED_PLAYLISTS,
            // translators: This is a sidebar entry that marks that the entries below are playlists.
            &gettext("Playlists"),
            "",
            true,
        ));
        list_store.append(&SideBarItem::new(
            NEW_PLAYLIST,
            // translators: This is a sidebar entry to create a new playlist.
            &gettext("New Playlist"),
            "list-add-symbolic",
            false,
        ));
        add_to_stack_and_listbox(
            &stack,
            &list_store,
            saved_playlists.get_root_widget(),
            SAVED_PLAYLISTS,
            // translators: This is a sidebar entry to browse to saved playlists.
            &gettext("All Playlists"),
            "view-app-grid-symbolic",
            false,
        );

        Self {
            model,
            stack,
            listbox,
            list_store,
            components: vec![
                Box::new(library),
                Box::new(saved_playlists),
                Box::new(saved_tracks),
                Box::new(now_playing),
            ],
            saved_playlists_model,
        }
    }

    pub fn connect_navigated<F: Fn() + 'static>(&self, f: F) {
        let playlist_model = self.saved_playlists_model.clone();
        self.listbox.connect_row_activated(
            clone!(@weak self.stack as stack @strong self.model as model => move |_, row| {
                let id = row.downcast_ref::<SideBarRow>().unwrap().id();
                match id.as_str() {
                    LIBRARY | SAVED_TRACKS | NOW_PLAYING | SAVED_PLAYLISTS => {
                        model.sidebar_item_activated(id.clone(), row.index());
                        stack.set_visible_child_name(&id);
                        f();
                    },
                    NEW_PLAYLIST => new_playlist_clicked(row, Rc::clone(&model)),
                    _ => {
                        model.sidebar_item_activated(id.clone(), row.index());
                        playlist_model.open_playlist(id);
                    },
                }
            }),
        );
    }

    pub fn connect_selected(&mut self) {
        self.listbox.connect_row_selected(
            clone!(@weak self.listbox as listbox @strong self.model as model => move |_, row| {
                let id = row.unwrap().downcast_ref::<SideBarRow>().unwrap().id();
                match id.as_str() {
                    SAVED_PLAYLISTS => {
                        if model.previously_selected_sidebar_item() != SAVED_PLAYLISTS {
                            model.reselect_currently_selected_row(listbox);
                        }
                    }
                    _ => {}
                }
            }),
        );
    }

    fn update_playlists_in_sidebar(&mut self) {
        let playlists = self.saved_playlists_model.get_playlists();
        let vec: Vec<SideBarItem> = playlists
            .iter()
            .take(NUM_PLAYLISTS)
            .map(make_playlist_item)
            .collect();
        self.list_store.splice(
            NUM_FIXED_ENTRIES,
            self.list_store.n_items() - NUM_FIXED_ENTRIES,
            vec.as_slice(),
        );
    }
}

impl Component for HomePane {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.stack.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.components)
    }
}

impl EventListener for HomePane {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::NowPlayingShown => {
                self.stack.set_visible_child_name("now_playing");
            }
            AppEvent::BrowserEvent(BrowserEvent::SavedPlaylistsUpdated) => {
                self.update_playlists_in_sidebar();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
