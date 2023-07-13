use gettextrs::gettext;
use glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::app::models::{PlaylistSummary, ArtistSummary};

const LIBRARY: &str = "library";
const SAVED_TRACKS: &str = "saved_tracks";
const NOW_PLAYING: &str = "now_playing";
const SAVED_PLAYLISTS: &str = "saved_playlists";
const PLAYLIST: &str = "playlist";
pub const SAVED_PLAYLISTS_SECTION: &str = "saved_playlists_section";
pub const CREATE_PLAYLIST_ITEM: &str = "create_playlist";
pub const FOLLOWED_ARTISTS_SECTION: &str = "followed_artists_section";
const ARTIST: &str = "artist";
const FOLLOWED_ARTISTS: &str = "followed_artists";

#[derive(Debug)]
pub enum SidebarDestination {
    Library,
    SavedTracks,
    NowPlaying,
    SavedPlaylists,
    Playlist(PlaylistSummary),
    Artist(ArtistSummary),
    FollowedArtists,
}

impl SidebarDestination {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Library => LIBRARY,
            Self::SavedTracks => SAVED_TRACKS,
            Self::NowPlaying => NOW_PLAYING,
            Self::SavedPlaylists => SAVED_PLAYLISTS,
            Self::Playlist(_) => PLAYLIST,
            Self::Artist(_) => ARTIST,
            Self::FollowedArtists => FOLLOWED_ARTISTS,
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
            Self::Artist(ArtistSummary {name, ..} ) => name.clone(),
            Self::FollowedArtists => gettext("Artists"),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Library => "library-music-symbolic",
            Self::SavedTracks => "starred-symbolic",
            Self::NowPlaying => "music-queue-symbolic",
            Self::SavedPlaylists => "view-app-grid-symbolic",
            Self::Playlist(_) => "playlist2-symbolic",
            Self::Artist(_) => "avatar-default-symbolic",
            Self::FollowedArtists => "view-app-grid-symbolic",
        }
    }
}

impl SidebarItem {
    pub fn from_destination(dest: SidebarDestination) -> Self {
        let (id, data, title) = match dest {
            SidebarDestination::Playlist(PlaylistSummary { id, title }) => {
                (PLAYLIST, Some(id), title)
            }
            SidebarDestination::Artist(ArtistSummary {id, name, ..}) => {
                (ARTIST, Some(id), name)
            }
            _ => (dest.id(), None, dest.title()),
        };
        glib::Object::builder()
            .property("id", id)
            .property("data", &data.unwrap_or_default())
            .property("title", &title)
            .property("navigatable", true)
            .build()
    }

    pub fn playlists_section() -> Self {
        glib::Object::builder()
            .property("id", SAVED_PLAYLISTS_SECTION)
            .property("data", &String::new())
            .property("title", &gettext("All Playlists"))
            .property("navigatable", false)
            .build()
    }

    pub fn create_playlist_item() -> Self {
        glib::Object::builder()
            .property("id", CREATE_PLAYLIST_ITEM)
            .property("data", &String::new())
            .property("title", &gettext("New Playlist"))
            .property("navigatable", false)
            .build()
    }

    pub fn artists_section() -> Self {
        glib::Object::builder()
            .property("id", FOLLOWED_ARTISTS_SECTION)
            .property("data", &String::new())
            .property("title", &gettext("Followed Artists"))
            .property("navigatable", false)
            .build()
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
                ARTIST => Some(SidebarDestination::Artist(ArtistSummary { id: data, name: title, photo: None })),
                FOLLOWED_ARTISTS => Some(SidebarDestination::FollowedArtists),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn icon(&self) -> Option<&str> {
        match self.id().as_str() {
            CREATE_PLAYLIST_ITEM => Some("list-add-symbolic"),
            _ => self.destination().map(|d| d.icon()),
        }
    }
}

mod imp {
    use super::*;
    use gdk::cairo::glib::ParamSpec;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SidebarItem)]
    pub struct SidebarItem {
        #[property(get, set)]
        pub id: RefCell<String>,
        #[property(get, set)]
        pub data: RefCell<String>,
        #[property(get, set)]
        pub title: RefCell<String>,
        #[property(get, set)]
        pub navigatable: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarItem {
        const NAME: &'static str = "SideBarItem";
        type Type = super::SidebarItem;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for SidebarItem {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct SidebarItem(ObjectSubclass<imp::SidebarItem>);
}
