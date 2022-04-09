use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::app::models::PlaylistSummary;

const LIBRARY: &str = "library";
const SAVED_TRACKS: &str = "saved_tracks";
const NOW_PLAYING: &str = "now_playing";
const SAVED_PLAYLISTS: &str = "saved_playlists";
const PLAYLIST: &str = "playlist";
pub const SAVED_PLAYLISTS_SECTION: &str = "saved_playlists_section";
pub const CREATE_PLAYLIST_ITEM: &str = "create_playlist";

#[derive(Debug)]
pub enum SidebarDestination {
    Library,
    SavedTracks,
    NowPlaying,
    SavedPlaylists,
    Playlist(PlaylistSummary),
}

impl SidebarDestination {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Library => LIBRARY,
            Self::SavedTracks => SAVED_TRACKS,
            Self::NowPlaying => NOW_PLAYING,
            Self::SavedPlaylists => SAVED_PLAYLISTS,
            Self::Playlist(_) => PLAYLIST,
        }
    }

    pub fn title(&self) -> String {
        match self {
            // translators: This is a sidebar entry to browse to saved albums.
            Self::Library => gettext("Library"),
            // translators: This is a sidebar entry to browse to saved tracks.
            Self::SavedTracks => gettext("Saved tracks"),
            // translators: This is a sidebar entry to browse to saved playlists.
            Self::NowPlaying => gettext("Now playing"),
            // translators: This is a sidebar entry that marks that the entries below are playlists.
            Self::SavedPlaylists => gettext("Playlists"),
            Self::Playlist(PlaylistSummary { title, .. }) => title.clone(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Library => "library-music-symbolic",
            Self::SavedTracks => "starred-symbolic",
            Self::NowPlaying => "music-queue-symbolic",
            Self::SavedPlaylists => "view-app-grid-symbolic",
            Self::Playlist(_) => "playlist2-symbolic",
        }
    }
}

impl SidebarItem {
    pub fn for_destination(dest: SidebarDestination) -> Self {
        let (id, data, title) = match dest {
            SidebarDestination::Playlist(PlaylistSummary { id, title }) => {
                (PLAYLIST, Some(id), title)
            }
            _ => (dest.id(), None, dest.title()),
        };
        glib::Object::new::<SidebarItem>(&[
            ("id", &id),
            ("data", &data.unwrap_or_default()),
            ("title", &title),
            ("navigatable", &true),
        ])
        .expect("Failed to create")
    }

    pub fn playlists_section() -> Self {
        glib::Object::new::<SidebarItem>(&[
            ("id", &SAVED_PLAYLISTS_SECTION),
            ("data", &String::new()),
            ("title", &gettext("All Playlists")),
            ("navigatable", &false),
        ])
        .expect("Failed to create")
    }

    pub fn create_playlist_item() -> Self {
        glib::Object::new::<SidebarItem>(&[
            ("id", &CREATE_PLAYLIST_ITEM),
            ("data", &String::new()),
            ("title", &gettext("New Playlist")),
            ("navigatable", &false),
        ])
        .expect("Failed to create")
    }

    pub fn destination(&self) -> Option<SidebarDestination> {
        let navigatable = self.property::<bool>("navigatable");
        if navigatable {
            let id = self.id();
            let data = self.property::<String>("data");
            let title = self.title();
            match id.as_str() {
                LIBRARY => Some(SidebarDestination::Library),
                SAVED_TRACKS => Some(SidebarDestination::SavedTracks),
                NOW_PLAYING => Some(SidebarDestination::NowPlaying),
                SAVED_PLAYLISTS => Some(SidebarDestination::SavedPlaylists),
                PLAYLIST => Some(SidebarDestination::Playlist(PlaylistSummary {
                    id: data,
                    title,
                })),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn id(&self) -> String {
        self.property::<String>("id")
    }

    pub fn title(&self) -> String {
        self.property::<String>("title")
    }

    pub fn icon(&self) -> Option<&str> {
        match self.id().as_str() {
            CREATE_PLAYLIST_ITEM => Some("list-add-symbolic"),
            _ => self.destination().map(|d| d.icon()),
        }
    }

    pub fn navigatable(&self) -> bool {
        self.property::<bool>("navigatable")
    }
}

mod imp {
    use super::*;
    use gdk::cairo::glib::ParamSpec;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct SidebarItem {
        pub id: RefCell<Option<String>>,
        pub data: RefCell<Option<String>>,
        pub title: RefCell<Option<String>>,
        pub navigatable: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarItem {
        const NAME: &'static str = "SideBarItem";
        type Type = super::SidebarItem;
        type ParentType = glib::Object;
    }

    lazy_static! {
        static ref PROPERTIES: [glib::ParamSpec; 4] = [
            glib::ParamSpecString::new("id", "ID", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpecString::new("data", "Data", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpecString::new("title", "Title", "", None, glib::ParamFlags::READWRITE),
            glib::ParamSpecBoolean::new(
                "navigatable",
                "Navigatable",
                "",
                false,
                glib::ParamFlags::READWRITE
            ),
        ];
    }

    impl ObjectImpl for SidebarItem {
        fn properties() -> &'static [ParamSpec] {
            &*PROPERTIES
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "id" => {
                    let id = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.id.replace(id);
                }
                "data" => {
                    let data = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.data.replace(data);
                }
                "title" => {
                    let title = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.title.replace(title);
                }
                "navigatable" => {
                    let navigatable = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.navigatable.replace(navigatable);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.borrow().to_value(),
                "data" => self.data.borrow().to_value(),
                "title" => self.title.borrow().to_value(),
                "navigatable" => self.navigatable.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SidebarItem(ObjectSubclass<imp::SidebarItem>);
}
