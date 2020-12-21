use gtk::prelude::*;
use gtk::{ButtonExt, ContainerExt};
use std::rc::Rc;

use crate::app::components::{EventListener, ListenerComponent, DetailsFactory, BrowserFactory, SearchFactory, ArtistDetailsFactory};
use crate::app::{AppEvent, BrowserEvent};
use crate::app::state::ScreenName;

use super::NavigationModel;

pub struct Navigation {
    model: Rc<NavigationModel>,
    stack: gtk::Stack,
    back_button: gtk::Button,
    browser_factory: BrowserFactory,
    details_factory: DetailsFactory,
    search_factory: SearchFactory,
    artist_details_factory: ArtistDetailsFactory,
    children: Vec<Box<dyn ListenerComponent>>
}

impl Navigation {

    pub fn new(
        model: NavigationModel,
        back_button: gtk::Button,
        stack: gtk::Stack,
        browser_factory: BrowserFactory,
        details_factory: DetailsFactory,
        search_factory: SearchFactory,
        artist_details_factory: ArtistDetailsFactory) -> Self {

        let model = Rc::new(model);
        let weak_model = Rc::downgrade(&model);
        back_button.connect_clicked(move |_| {
            weak_model.upgrade().map(|m| m.go_back());
        });

        Self {
            model, stack, back_button,
            browser_factory, details_factory, search_factory, artist_details_factory,
            children: vec![]
        }
    }


    fn push_screen(&mut self, name: &ScreenName) {
        println!("{}", name.identifier());

        let component: Box<dyn ListenerComponent> = match name {
            ScreenName::Library => Box::new(self.browser_factory.make_browser()),
            ScreenName::Details(id) => Box::new(self.details_factory.make_details(id.to_owned())),
            ScreenName::Search => Box::new(self.search_factory.make_search_results()),
            ScreenName::Artist(id) => Box::new(self.artist_details_factory.make_artist_details(id.to_owned()))
        };

        let widget = component.get_root_widget();
        widget.show_all();

        self.stack.add_named(widget, name.identifier().as_ref());
        self.children.push(component);
        self.stack.set_visible_child_name(name.identifier().as_ref());
    }


    fn pop(&mut self) {
        let children = &mut self.children;
        let popped = children.pop();

        let name = self.model.visible_child_name();
        self.stack.set_visible_child_name(name.identifier().as_ref());

        if let Some(child) = popped {
            self.stack.remove(child.get_root_widget());
        }
    }

    fn pop_to(&mut self, screen: &ScreenName) {
        self.stack.set_visible_child_name(screen.identifier().as_ref());
        let remainder = self.children.split_off(self.model.children_count());
        for widget in remainder {
            self.stack.remove(widget.get_root_widget());
        }
    }

    fn update_back_button(&self) {
        self.back_button.set_sensitive(self.model.can_go_back());
    }
}

impl EventListener for Navigation {

    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                self.push_screen(&ScreenName::Library);
                self.update_back_button();
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPushed(name)) => {
                self.push_screen(name);
                self.update_back_button();
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.pop();
                self.update_back_button();
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPoppedTo(name)) => {
                self.pop_to(name);
                self.update_back_button();
            }
            _ => {}
        };
        if let Some(listener) = self.children.iter_mut().last() {
            listener.on_event(event);
        }
    }
}
