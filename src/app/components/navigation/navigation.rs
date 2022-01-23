use gtk::traits::WidgetExt;
use libadwaita::NavigationDirection;
use std::rc::Rc;

use crate::app::components::{EventListener, ListenerComponent};
use crate::app::state::ScreenName;
use crate::app::{AppEvent, BrowserEvent};

use super::{factory::ScreenFactory, home::HomePane, NavigationModel};

pub struct Navigation {
    model: Rc<NavigationModel>,
    leaflet: libadwaita::Leaflet,
    navigation_stack: gtk::Stack,
    home_listbox: gtk::ListBox,
    home_list_store: gio::ListStore,
    screen_factory: ScreenFactory,
    children: Vec<Box<dyn ListenerComponent>>,
}

impl Navigation {
    pub fn new(
        model: NavigationModel,
        leaflet: libadwaita::Leaflet,
        navigation_stack: gtk::Stack,
        home_listbox: gtk::ListBox,
        home_list_store: gio::ListStore,
        screen_factory: ScreenFactory,
    ) -> Self {
        let model = Rc::new(model);

        leaflet.connect_folded_notify(
            clone!(@weak model => move |leaflet| {
                let is_main = leaflet.visible_child_name().map(|s| s.as_str() == "main").unwrap_or(false);
                let folded = leaflet.is_folded();
                model.set_nav_hidden(folded && is_main);
            })
        );

        leaflet.connect_visible_child_name_notify(
            clone!(@weak model => move |leaflet| {
                let is_main = leaflet.visible_child_name().map(|s| s.as_str() == "main").unwrap_or(false);
                let folded = leaflet.is_folded();
                model.set_nav_hidden(folded && is_main);
            })
        );

        Self {
            model,
            leaflet,
            navigation_stack,
            home_listbox,
            home_list_store,
            screen_factory,
            children: vec![],
        }
    }

    fn make_home(&self) -> Box<dyn ListenerComponent> {
        let mut home = HomePane::new(
            self.home_listbox.clone(),
            &self.screen_factory,
            self.home_list_store.clone(),
            clone!(@weak self.model as model, @weak self.leaflet as leaflet => move || {
                leaflet.navigate(NavigationDirection::Forward);
                model.go_home();
            }
                ),
        );

        home.connect_navigated();

        Box::new(home)
    }

    fn show_navigation(&self) {
        self.leaflet.navigate(NavigationDirection::Back);
    }

    fn push_screen(&mut self, name: &ScreenName) {
        let component: Box<dyn ListenerComponent> = match name {
            ScreenName::Home => self.make_home(),
            ScreenName::AlbumDetails(id) => {
                Box::new(self.screen_factory.make_album_details(id.to_owned()))
            }
            ScreenName::Search => Box::new(self.screen_factory.make_search_results()),
            ScreenName::Artist(id) => {
                Box::new(self.screen_factory.make_artist_details(id.to_owned()))
            }
            ScreenName::PlaylistDetails(id) => {
                Box::new(self.screen_factory.make_playlist_details(id.to_owned()))
            }
            ScreenName::User(id) => Box::new(self.screen_factory.make_user_details(id.to_owned())),
        };

        let widget = component.get_root_widget().clone();
        self.children.push(component);

        self.leaflet.navigate(NavigationDirection::Forward);
        self.navigation_stack
            .add_named(&widget, Some(name.identifier().as_ref()));
        self.navigation_stack
            .set_visible_child_name(name.identifier().as_ref());

        glib::source::idle_add_local_once(move || {
            widget.grab_focus();
        });
    }

    fn pop(&mut self) {
        let children = &mut self.children;
        let popped = children.pop();

        let name = self.model.visible_child_name();
        self.navigation_stack
            .set_visible_child_name(name.identifier().as_ref());

        if let Some(child) = popped {
            self.navigation_stack.remove(child.get_root_widget());
        }
    }

    fn pop_to(&mut self, screen: &ScreenName) {
        self.navigation_stack
            .set_visible_child_name(screen.identifier().as_ref());
        let remainder = self.children.split_off(self.model.children_count());
        for widget in remainder {
            self.navigation_stack.remove(widget.get_root_widget());
        }
    }
}

impl EventListener for Navigation {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                self.push_screen(&ScreenName::Home);
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPushed(name)) => {
                self.push_screen(name);
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationHidden(false)) => {
                self.show_navigation();
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.pop();
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPoppedTo(name)) => {
                self.pop_to(name);
            }
            _ => {}
        };
        for child in self.children.iter_mut() {
            child.on_event(event);
        }
    }
}
