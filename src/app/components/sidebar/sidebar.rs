use gettextrs::gettext;
use gtk::prelude::*;
use std::rc::Rc;

use super::FOLLOWED_ARTISTS_SECTION;
use super::create_playlist::CreatePlaylistPopover;
use super::{
    sidebar_row::SidebarRow, SidebarDestination, SidebarItem, CREATE_PLAYLIST_ITEM,
    SAVED_PLAYLISTS_SECTION,
};
use crate::app::models::{AlbumModel, PlaylistSummary, ArtistSummary, ArtistModel};
use crate::app::state::ScreenName;
use crate::app::{
    ActionDispatcher, AppAction, AppEvent, AppModel, BrowserAction, BrowserEvent, Component,
    EventListener,
};

const NUM_FIXED_ENTRIES: u32 = 6;
const NUM_PLAYLISTS: usize = 20;
const NUM_ARTISTS: usize = 20;

pub struct SidebarModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl SidebarModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn get_playlists(&self) -> Vec<SidebarDestination> {
        self.app_model
            .get_state()
            .browser
            .home_state()
            .expect("expected HomeState to be available")
            .playlists
            .iter()
            .take(NUM_PLAYLISTS)
            .map(Self::map_playlist_to_destination)
            .collect()
    }

    fn map_playlist_to_destination(a: AlbumModel) -> SidebarDestination {
        let title = Some(a.album())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| gettext("Unnamed playlist"));
        let id = a.uri();
        SidebarDestination::Playlist(PlaylistSummary { id, title })
    }

    fn create_new_playlist(&self, name: String) {
        let user_id = self.app_model.get_state().logged_user.user.clone().unwrap();
        let api = self.app_model.get_spotify();
        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.create_new_playlist(name.as_str(), user_id.as_str())
                    .await
                    .map(AppAction::CreatePlaylist)
            })
    }

    fn get_followed_artists(&self) -> Vec<SidebarDestination> {
        self.app_model
            .get_state()
            .browser
            .home_state()
            .expect("expected HomeState to be available")
            .followed_artists
            .iter()
            .take(NUM_ARTISTS)
            .map(Self::map_artist_to_destination)
            .collect()
    }

    fn map_artist_to_destination(a: ArtistModel) -> SidebarDestination {
        let name = Some(a.artist())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| gettext("Unnamed artist"));
        let id = a.id();
        SidebarDestination::Artist(ArtistSummary { id, name , photo: None })
    }

    fn navigate(&self, dest: SidebarDestination) {
        let actions = match dest {
            SidebarDestination::Library
            | SidebarDestination::SavedTracks
            | SidebarDestination::NowPlaying
            | SidebarDestination::SavedPlaylists 
            | SidebarDestination::FollowedArtists => {
                vec![
                    BrowserAction::NavigationPopTo(ScreenName::Home).into(),
                    BrowserAction::SetHomeVisiblePage(dest.id()).into(),
                ]
            }
            SidebarDestination::Playlist(PlaylistSummary { id, .. }) => {
                vec![AppAction::ViewPlaylist(id)]
            },
            SidebarDestination::Artist(ArtistSummary {id, ..}) => {
                vec![AppAction::ViewArtist(id)]
            }
        };
        self.dispatcher.dispatch_many(actions);
    }
}

pub struct Sidebar {
    listbox: gtk::ListBox,
    list_store: gio::ListStore,
    model: Rc<SidebarModel>,
}

impl Sidebar {
    pub fn new(listbox: gtk::ListBox, model: Rc<SidebarModel>) -> Self {
        let popover = CreatePlaylistPopover::new();
        popover.connect_create(clone!(@weak model => move |t| model.create_new_playlist(t)));

        let list_store = gio::ListStore::new(SidebarItem::static_type());

        list_store.append(&SidebarItem::from_destination(SidebarDestination::Library));
        list_store.append(&SidebarItem::from_destination(
            SidebarDestination::SavedTracks,
        ));
        list_store.append(&SidebarItem::from_destination(
            SidebarDestination::NowPlaying,
        ));
        list_store.append(&SidebarItem::playlists_section());
        list_store.append(&SidebarItem::create_playlist_item());
        list_store.append(&SidebarItem::from_destination(
            SidebarDestination::SavedPlaylists,
        ));        
        list_store.append(&SidebarItem::artists_section());

        listbox.bind_model(
            Some(&list_store),
            clone!(@weak popover => @default-panic, move |obj| {
                let item = obj.downcast_ref::<SidebarItem>().unwrap();
                if item.navigatable() {
                    Self::make_navigatable(item)
                } else {
                    match item.id().as_str() {
                        SAVED_PLAYLISTS_SECTION => Self::make_section_label(item),
                        CREATE_PLAYLIST_ITEM => Self::make_create_playlist(item, popover),
                        FOLLOWED_ARTISTS_SECTION => Self::make_section_label(item),
                        _ => unimplemented!(),
                    }
                }
            }),
        );

        listbox.connect_row_activated(clone!(@weak popover, @weak model => move |_, row| {
            if let Some(row) = row.downcast_ref::<SidebarRow>() {
                if let Some(dest) = row.item().destination() {
                    model.navigate(dest);
                } else {
                    match row.item().id().as_str() {
                        CREATE_PLAYLIST_ITEM => popover.popup(),
                        _ => unimplemented!()
                    }
                }
            }
        }));

        Self {
            listbox,
            list_store,
            model,
        }
    }

    fn make_navigatable(item: &SidebarItem) -> gtk::Widget {
        let row = SidebarRow::new(item.clone());
        row.set_selectable(false);
        row.upcast()
    }

    fn make_section_label(item: &SidebarItem) -> gtk::Widget {
        let label = gtk::Label::new(Some(item.title().as_str()));
        label.add_css_class("caption-heading");
        let row = gtk::ListBoxRow::builder()
            .activatable(false)
            .selectable(false)
            .sensitive(false)
            .child(&label)
            .build();
        row.upcast()
    }

    fn make_create_playlist(item: &SidebarItem, popover: CreatePlaylistPopover) -> gtk::Widget {
        let row = SidebarRow::new(item.clone());
        row.set_activatable(true);
        row.set_selectable(false);
        row.set_sensitive(true);
        popover.set_parent(&row);
        row.upcast()
    }

    fn update_playlists_and_followed_artists_in_sidebar(&self) {
        let playlists: Vec<SidebarItem> = self
            .model
            .get_playlists()
            .into_iter()
            .map(SidebarItem::from_destination)
            .collect();
        let artists: Vec<SidebarItem> = self
            .model
            .get_followed_artists()
            .into_iter()
            .map(SidebarItem::from_destination)
            .collect();
        self.list_store.splice(
            NUM_FIXED_ENTRIES,
            self.list_store.n_items() - NUM_FIXED_ENTRIES,
            playlists.as_slice(),
        );
        self.list_store.splice(
            self.list_store.n_items(),
            0,
            artists.as_slice(),
        );

        // TODO fix this mess
        // Layout:
        // FIXED ENTRIES (6)
        // PLAYLISTS
        // Artists title
        // ARTISTS
    }
}

impl Component for Sidebar {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.listbox.upcast_ref()
    }
}

impl EventListener for Sidebar {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::SavedPlaylistsUpdated | BrowserEvent::FollowedArtistsUpdated) => self.update_playlists_and_followed_artists_in_sidebar(),
            _ => ()
        }
    }
}
