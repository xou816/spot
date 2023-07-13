use gtk::prelude::*;

use crate::app::components::sidebar::SidebarDestination;
use crate::app::components::{Component, EventListener, ScreenFactory};
use crate::app::{AppEvent, BrowserEvent};

pub struct HomePane {
    stack: gtk::Stack,
    components: Vec<Box<dyn EventListener>>,
}

impl HomePane {
    pub fn new(listbox: gtk::ListBox, screen_factory: &ScreenFactory) -> Self {
        let library = screen_factory.make_library();
        let saved_playlists = screen_factory.make_saved_playlists();
        let saved_tracks = screen_factory.make_saved_tracks();
        let now_playing = screen_factory.make_now_playing();
        let followed_artists = screen_factory.make_followed_artists();
        let sidebar = screen_factory.make_sidebar(listbox);

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);

        let dest = SidebarDestination::Library;
        stack.add_titled(
            library.get_root_widget(),
            Option::from(dest.id()),
            &dest.title(),
        );

        let dest = SidebarDestination::SavedTracks;
        stack.add_titled(
            saved_tracks.get_root_widget(),
            Option::from(dest.id()),
            &dest.title(),
        );

        let dest = SidebarDestination::SavedPlaylists;
        stack.add_titled(
            saved_playlists.get_root_widget(),
            Option::from(dest.id()),
            &dest.title(),
        );

        let dest = SidebarDestination::FollowedArtists;
        stack.add_titled(
            followed_artists.get_root_widget(),
            Option::from(dest.id()),
            &dest.title(),
        );

        let dest = SidebarDestination::NowPlaying;
        stack.add_titled(
            now_playing.get_root_widget(),
            Option::from(dest.id()),
            &dest.title(),
        );

        Self {
            stack,
            components: vec![
                Box::new(sidebar),
                Box::new(library),
                Box::new(saved_playlists),
                Box::new(saved_tracks),
                Box::new(now_playing),
                Box::new(followed_artists),
            ],
        }
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
                self.stack
                    .set_visible_child_name(SidebarDestination::NowPlaying.id());
            }
            AppEvent::BrowserEvent(BrowserEvent::HomeVisiblePageChanged(page)) => {
                self.stack.set_visible_child_name(page);
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
