use gtk::prelude::WidgetExt;
use std::rc::Rc;

use crate::app::components::{EventListener, ListenerComponent};
use crate::app::state::ScreenName;
use crate::app::{AppEvent, BrowserEvent};

use super::{factory::ScreenFactory, home::HomePane, NavigationModel};

pub struct Navigation {
    model: Rc<NavigationModel>,
    split_view: libadwaita::NavigationSplitView,
    navigation_stack: gtk::Stack,
    home_listbox: gtk::ListBox,
    screen_factory: ScreenFactory,
    children: Vec<Box<dyn ListenerComponent>>,
}

impl Navigation {
    pub fn new(
        model: NavigationModel,
        split_view: libadwaita::NavigationSplitView,
        navigation_stack: gtk::Stack,
        home_listbox: gtk::ListBox,
        screen_factory: ScreenFactory,
    ) -> Self {
        let model = Rc::new(model);

        split_view.connect_collapsed_notify(clone!(
            #[weak]
            model,
            move |split_view| {
                let is_main = split_view.shows_content();
                let folded = split_view.is_collapsed();
                if folded {
                    split_view.add_css_class("collapsed");
                } else {
                    split_view.remove_css_class("collapsed");
                }
                model.set_nav_hidden(folded && is_main);
            }
        ));

        split_view.connect_show_content_notify(clone!(
            #[weak]
            model,
            move |split_view| {
                let is_main = split_view.shows_content();
                let folded = split_view.is_collapsed();
                if folded {
                    split_view.add_css_class("collapsed");
                } else {
                    split_view.remove_css_class("collapsed");
                }
                model.set_nav_hidden(folded && is_main);
            }
        ));

        Self {
            model,
            split_view,
            navigation_stack,
            home_listbox,
            screen_factory,
            children: vec![],
        }
    }

    fn make_home(&self) -> Box<dyn ListenerComponent> {
        Box::new(HomePane::new(
            self.home_listbox.clone(),
            &self.screen_factory,
        ))
    }

    fn show_navigation(&self) {
        self.split_view.set_show_content(false);
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

        self.split_view.set_show_content(true);
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
            AppEvent::BrowserEvent(BrowserEvent::HomeVisiblePageChanged(_)) => {
                self.split_view.set_show_content(true);
            }
            _ => {}
        };
        for child in self.children.iter_mut() {
            child.on_event(event);
        }
    }
}
