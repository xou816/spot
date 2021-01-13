use gtk::prelude::*;
use gtk::{ButtonExt, ContainerExt, StackExt, StackSidebarExt};
use std::rc::Rc;

use crate::app::components::{
    ArtistDetailsFactory, Browser, BrowserFactory, DetailsFactory, EventListener,
    ListenerComponent, SearchFactory,
};
use crate::app::state::ScreenName;
use crate::app::{AppEvent, BrowserEvent};

use super::{home::HomeComponent, NavigationModel};

pub struct Navigation {
    model: Rc<NavigationModel>,
    navigation_stack: gtk::Stack,
    home_stack_sidebar: gtk::StackSidebar,
    back_button: gtk::Button,
    browser_factory: BrowserFactory,
    details_factory: DetailsFactory,
    search_factory: SearchFactory,
    artist_details_factory: ArtistDetailsFactory,
    children: Vec<Box<dyn ListenerComponent>>,
}

impl Navigation {
    pub fn new(
        model: NavigationModel,
        back_button: gtk::Button,
        navigation_stack: gtk::Stack,
        home_stack_sidebar: gtk::StackSidebar,
        browser_factory: BrowserFactory,
        details_factory: DetailsFactory,
        search_factory: SearchFactory,
        artist_details_factory: ArtistDetailsFactory,
    ) -> Self {
        let model = Rc::new(model);
        let weak_model = Rc::downgrade(&model);
        back_button.connect_clicked(move |_| {
            if let Some(m) = weak_model.upgrade() {
                m.go_back()
            }
        });

        Self {
            model,
            navigation_stack,
            home_stack_sidebar,
            back_button,
            browser_factory,
            details_factory,
            search_factory,
            artist_details_factory,
            children: vec![],
        }
    }

    fn make_home(&self) -> Box<dyn ListenerComponent> {
        let home = HomeComponent::new(
            self.home_stack_sidebar.clone(),
            self.browser_factory.make_browser(),
        );

        let weak_model = Rc::downgrade(&self.model);
        home.connect_navigated(move || {
            if let Some(m) = weak_model.upgrade() {
                m.go_home();
            }
        });

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

    fn update_back_button(&self) {
        self.back_button.set_sensitive(self.model.can_go_back());
    }
}

impl EventListener for Navigation {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                self.push_screen(&ScreenName::Home);
                self.update_back_button();
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPushed(name)) => {
                self.push_screen(name);
                self.update_back_button();
            }
            AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.pop();
                self.update_back_button();
            }
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
