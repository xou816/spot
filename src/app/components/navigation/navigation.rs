use gtk::prelude::*;
use gtk::{ButtonExt, ContainerExt};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::components::{EventListener, ListenerComponent, DetailsFactory, BrowserFactory, SearchFactory};
use crate::app::{AppEvent, BrowserEvent};

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
    children: RefCell<Vec<Box<dyn ListenerComponent>>>
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
        let browser = self.browser_factory.make_browser();
        self.add_component(Box::new(browser), "library");
    }

    fn create_details(&self) {
        let details = self.details_factory.make_details();
        self.add_component(Box::new(details), "details");
    }

    fn create_search(&self) {
        let search_results = self.search_factory.make_search_results();
        self.add_component(Box::new(search_results), "search")
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
            AppEvent::BrowserEvent(BrowserEvent::NavigatedToSearch) => {
                self.create_search();
                self.switch_to("search");
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.pop();
            }
            _ => {}
        };
        self.broadcast(event);
    }
}
