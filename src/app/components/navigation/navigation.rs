use gtk::prelude::*;
use gtk::{ButtonExt, ContainerExt};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::components::{EventListener, ListenerComponent, DetailsFactory, BrowserFactory, SearchFactory};
use crate::app::{AppEvent, BrowserEvent};
use crate::app::state::ScreenName;

struct NamedWidget(pub ScreenName, pub Box<dyn ListenerComponent>);

pub trait NavigationModel {
    fn go_back(&self);
    fn can_go_back(&self) -> bool;
}

pub struct Navigation {
    model: Rc<dyn NavigationModel>,
    stack: gtk::Stack,
    browser_factory: BrowserFactory,
    details_factory: DetailsFactory,
    search_factory: SearchFactory,
    children: RefCell<Vec<NamedWidget>>
}

impl Navigation {

    pub fn new(
        model: Rc<dyn NavigationModel>,
        back_button: gtk::Button,
        stack: gtk::Stack,
        browser_factory: BrowserFactory,
        details_factory: DetailsFactory,
        search_factory: SearchFactory) -> Self {

        let weak_model = Rc::downgrade(&model);
        back_button.connect_clicked(move |_| {
            weak_model.upgrade().map(|m| m.go_back());
        });

        Self { model, stack, browser_factory, details_factory, search_factory, children: RefCell::new(vec![]) }
    }


    fn push_screen(&self, name: &ScreenName) {
        let component: Box<dyn ListenerComponent> = match name {
            ScreenName::Library => Box::new(self.browser_factory.make_browser()),
            ScreenName::Details(_) => Box::new(self.details_factory.make_details()),
            ScreenName::Search => Box::new(self.search_factory.make_search_results())
        };

        let widget = component.get_root_widget();
        widget.show_all();

        self.stack.add_named(widget, name.identifier());
        self.children.borrow_mut().push(NamedWidget(name.clone(), component));
        self.stack.set_visible_child_name(name.identifier());
    }


    fn pop(&self) {
        let mut children = self.children.borrow_mut();
        let popped = children.pop();
        if let Some(NamedWidget(name, _)) = children.last() {
            self.stack.set_visible_child_name(name.identifier());
        }
        if let Some(NamedWidget(_, child)) = popped {
            self.stack.remove(child.get_root_widget());
        }
    }

    fn pop_to(&self, screen: &ScreenName) {
        self.stack.set_visible_child_name(screen.identifier());
        let mut children = self.children.borrow_mut();
        let i = children
            .iter()
            .position(|NamedWidget(name, _)| name == screen)
            .unwrap();
        let remainder = children.split_off(i + 1);
        for NamedWidget(_, widget) in remainder {
            self.stack.remove(widget.get_root_widget());
        }
    }
}

impl EventListener for Navigation {

    fn on_event(&self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                self.push_screen(&ScreenName::Library);
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPushed(name)) => {
                self.push_screen(name);
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.pop();
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPoppedTo(name)) => {
                self.pop_to(name);
            }
            _ => {}
        };
        if let Some(NamedWidget(_, listener)) = self.children.borrow().iter().last() {
            listener.on_event(event);
        }
    }
}
