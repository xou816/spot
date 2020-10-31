use gtk::prelude::*;
use gtk::{ButtonExt, ContainerExt};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::connect::{PlaylistFactory, BrowserFactory};
use crate::app::components::{EventListener, ListenerComponent, Details};
use crate::app::{AppEvent, BrowserEvent};

use super::browser::{Browser};

pub trait NavigationModel {
    fn go_back(&self);
    fn can_go_back(&self) -> bool;
}

pub struct Navigation {
    model: Rc<dyn NavigationModel>,
    stack: gtk::Stack,
    browser_factory: BrowserFactory,
    playlist_factory: PlaylistFactory,
    children: RefCell<Vec<Box<dyn ListenerComponent>>>
}

impl Navigation {

    pub fn new(
        model: Rc<dyn NavigationModel>,
        back_button: gtk::Button,
        stack: gtk::Stack,
        browser_factory: BrowserFactory,
        playlist_factory: PlaylistFactory) -> Self {

        let weak_model = Rc::downgrade(&model);
        back_button.connect_clicked(move |_| {
            weak_model.upgrade().map(|m| m.go_back());
        });

        Self { model, stack, browser_factory, playlist_factory, children: RefCell::new(vec![]) }
    }

    fn add_component(&self, component: Box<dyn ListenerComponent>, name: &'static str) {
        let widget = component.get_root_widget();
        widget.show_all();
        self.stack.add_named(widget, name);

        self.children.borrow_mut().push(component);
    }

    fn broadcast(&self, event: &AppEvent) {
        for child in self.children.borrow().iter() {
            child.on_event(event);
        }
    }

    fn switch_to(&self, name: &'static str) {
        self.stack.set_visible_child_name(name);
    }

    fn create_browser(&self) {
        let browser: Box<Browser> = Box::new(self.browser_factory.make_browser());
        self.add_component(browser, "library");
    }

    fn create_details(&self) {
        let details: Box<Details> = Box::new(Details::new(&self.playlist_factory));
        self.add_component(details, "details");
    }

    fn pop(&self) {
        let mut children = self.children.borrow_mut();
        let popped = children.pop();
        if let Some(last) = children.last() {
            self.stack.set_visible_child(last.get_root_widget())
        }
        if let Some(child) = popped {
            self.stack.remove(child.get_root_widget());
        }
    }
}

impl EventListener for Navigation {

    fn on_event(&self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                self.create_browser();
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigatedToDetails) => {
                self.create_details();
                self.switch_to("details");
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.pop();
            },
            _ => {}
        };
        self.broadcast(event);
    }
}
