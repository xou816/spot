use gtk::prelude::*;
use gtk::{ButtonExt, ContainerExt};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::components::{EventListener, ListenerComponent, DetailsFactory, BrowserFactory, SearchFactory};
use crate::app::{AppEvent, BrowserEvent};

struct NamedWidget(pub String, pub Box<dyn ListenerComponent>);

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

    fn add_component(&self, component: Box<dyn ListenerComponent>, name: String) {
        let widget = component.get_root_widget();
        widget.show_all();
        self.stack.add_named(widget, &name);
        self.children.borrow_mut().push(NamedWidget(name, component));
    }

    fn switch_to(&self, name: &str) {
        self.stack.set_visible_child_name(name);
    }

    fn create_browser(&self) -> &str {
        let name = "library";
        let browser = self.browser_factory.make_browser();
        self.add_component(Box::new(browser), name.to_owned());
        name
    }

    fn create_details(&self, name: String) {
        let details = self.details_factory.make_details();
        self.add_component(Box::new(details), name);
    }

    fn create_search(&self) -> &str {
        let name = "search";
        let search_results = self.search_factory.make_search_results();
        self.add_component(Box::new(search_results), name.to_owned());
        name
    }

    fn pop(&self) {
        let mut children = self.children.borrow_mut();
        let popped = children.pop();
        if let Some(NamedWidget(name, _)) = children.last() {
            self.stack.set_visible_child_name(name);
        }
        if let Some(NamedWidget(_, child)) = popped {
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
            AppEvent::BrowserEvent(BrowserEvent::NavigatedToDetails(tag)) => {
                self.create_details(tag.clone());
                self.switch_to(tag);
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigatedToSearch) => {
                let name = self.create_search();
                self.switch_to(name);
            },
            AppEvent::BrowserEvent(BrowserEvent::NavigationPopped) => {
                self.pop();
            }
            _ => {}
        };
        if let Some(NamedWidget(_, listener)) = self.children.borrow().iter().last() {
            listener.on_event(event);
        }
    }
}
