use gettextrs::*;
use gtk::prelude::*;

use crate::app::components::sidebar_listbox::SideBarItem;
use crate::app::components::{Component, EventListener, SavedPlaylistsModel, ScreenFactory};
use crate::app::models::AlbumModel;
use crate::app::{AppEvent, BrowserEvent};

const LIBRARY: &str = "library";
const SAVED_TRACKS: &str = "saved_tracks";
const NOW_PLAYING: &str = "now_playing";
const SAVED_PLAYLISTS: &str = "saved_playlists";
const NUM_FIXED_ENTRIES: u32 = 5;
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
    let mut title = playlist_item.album_title().unwrap();
    if title.is_empty() {
        title = gettext("Unnamed playlist");
    }

    let id = playlist_item.uri().unwrap();

    SideBarItem::new(id.as_str(), &title, "playlist2-symbolic", false)
}

pub struct HomePane {
    stack: gtk::Stack,
    listbox: gtk::ListBox,
    list_store: gio::ListStore,
    components: Vec<Box<dyn EventListener>>,
    saved_playlists_model: SavedPlaylistsModel,
}

impl HomePane {
    pub fn new(
        listbox: gtk::ListBox,
        screen_factory: &ScreenFactory,
        list_store: gio::ListStore,
    ) -> Self {
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
        let model = self.saved_playlists_model.box_clone();
        self.listbox
            .connect_row_activated(clone!(@weak self.stack as stack => move |_, row| {
                let n = row.property("id").expect("Could not get id of ListBoxRow.");
                let name = n.get::<&str>().expect("Could not get id of ListBoxRow.");
                match name {
                    LIBRARY | SAVED_TRACKS | NOW_PLAYING | SAVED_PLAYLISTS => {
                        stack.set_visible_child_name(name);
                        f();
                    },
                    _ => model.open_playlist(name.to_string()),
                }
            }));
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
                let mut vec = Vec::new();
                let playlists = self.saved_playlists_model.get_playlists();
                for (i, playlist_item) in playlists.iter().enumerate() {
                    if i == NUM_PLAYLISTS {
                        break;
                    }
                    vec.push(make_playlist_item(playlist_item).upcast());
                }
                self.list_store.splice(
                    NUM_FIXED_ENTRIES,
                    self.list_store.n_items() - NUM_FIXED_ENTRIES,
                    vec.as_slice(),
                );
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
