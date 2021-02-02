use gtk::prelude::*;
use gtk::{ButtonExt, ContainerExt, StackExt};
use libhandy::LeafletExt;
use libhandy::NavigationDirection;
use std::rc::Rc;

use crate::app::components::{
    ArtistDetailsFactory, BrowserFactory, DetailsFactory, EventListener, ListenerComponent,
    NowPlayingFactory, SearchFactory,
};
use crate::app::state::ScreenName;
use crate::app::{AppEvent, BrowserEvent};

use super::{home::HomeComponent, NavigationModel};

pub struct Navigation {
    model: Rc<NavigationModel>,
    leaflet: libhandy::Leaflet,
    navigation_stack: gtk::Stack,
    home_stack_sidebar: gtk::StackSidebar,
    back_button: gtk::Button,
    browser_factory: BrowserFactory,
    details_factory: DetailsFactory,
    search_factory: SearchFactory,
    artist_details_factory: ArtistDetailsFactory,
    now_playing_factory: NowPlayingFactory,
    children: Vec<Box<dyn ListenerComponent>>,
}

impl Navigation {
    pub fn new(
        model: NavigationModel,
        leaflet: libhandy::Leaflet,
        back_button: gtk::Button,
        navigation_stack: gtk::Stack,
        home_stack_sidebar: gtk::StackSidebar,
        browser_factory: BrowserFactory,
        details_factory: DetailsFactory,
        search_factory: SearchFactory,
        artist_details_factory: ArtistDetailsFactory,
        now_playing_factory: NowPlayingFactory,
    ) -> Self {
        let model = Rc::new(model);

        Self::connect_back_button(&back_button, &leaflet, &model);

        leaflet.connect_property_folded_notify(
            clone!(@weak back_button, @weak model => move |leaflet| {
                Self::update_back_button(&back_button, &leaflet, &model);
            }),
        );

        Self {
            model,
            leaflet,
            navigation_stack,
            home_stack_sidebar,
            back_button,
            browser_factory,
            details_factory,
            search_factory,
            artist_details_factory,
            now_playing_factory,
            children: vec![],
        }
    }

    fn update_back_button(
        back_button: &gtk::Button,
        leaflet: &libhandy::Leaflet,
        model: &Rc<NavigationModel>,
    ) {
        back_button.set_sensitive(leaflet.get_folded() || model.can_go_back());
    }

    fn connect_back_button(
        back_button: &gtk::Button,
        leaflet: &libhandy::Leaflet,
        model: &Rc<NavigationModel>,
    ) {
        back_button.connect_clicked(clone!(@weak leaflet, @weak model => move |_| {
            let folded = leaflet.get_folded();
            let can_go_back = model.can_go_back();
            match (folded, can_go_back) {
                (_, true) => {
                    model.go_back();
                }
                (true, false) => {
                    leaflet.navigate(NavigationDirection::Back);
                }
                (false, false) => {}
            }
        }));
    }

    fn make_home(&self) -> Box<dyn ListenerComponent> {
        let home = HomeComponent::new(
            self.home_stack_sidebar.clone(),
            self.browser_factory.make_browser(),
            self.now_playing_factory.make_now_playing(),
        );

        home.connect_navigated(
            clone!(@weak self.model as model, @weak self.leaflet as leaflet => move || {
                leaflet.navigate(NavigationDirection::Forward);
                model.go_home();
            }),
        );

        Box::new(home)
    }

    fn push_screen(&mut self, name: &ScreenName) {
        println!("{}", name.identifier());

        let component: Box<dyn ListenerComponent> = match name {
            ScreenName::Home => self.make_home(),
            ScreenName::Details(id) => Box::new(self.details_factory.make_details(id.to_owned())),
            ScreenName::Search => Box::new(self.search_factory.make_search_results()),
            ScreenName::Artist(id) => Box::new(
                self.artist_details_factory
                    .make_artist_details(id.to_owned()),
            ),
        };

        let widget = component.get_root_widget();
        widget.show_all();

        self.navigation_stack
            .add_named(widget, name.identifier().as_ref());
        self.children.push(component);
        self.navigation_stack
            .set_visible_child_name(name.identifier().as_ref());
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

    fn do_update_back_button(&self) {
        Self::update_back_button(&self.back_button, &self.leaflet, &self.model);
    }
}

impl EventListener for Navigation {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                self.push_screen(&ScreenName::Home);
                self.do_update_back_button();
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPushed(name)) => {
                self.push_screen(name);
                self.do_update_back_button();
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.pop();
                self.do_update_back_button();
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPoppedTo(name)) => {
                self.pop_to(name);
                self.do_update_back_button();
            }
            _ => {}
        };
        for child in self.children.iter_mut() {
            child.on_event(event);
        }
    }
}
