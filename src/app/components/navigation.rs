use gtk::prelude::*;
use std::collections::HashMap;
use std::cell::RefCell;
use std::convert::Into;

use crate::app::connect::BrowserFactory;
use crate::app::components::Component;
use crate::app::AppEvent;

use super::browser::{Browser};

enum NavigationPage {
    Home
}

impl Into<&'static str> for NavigationPage {
    fn into(self) -> &'static str {
        match self {
            Self::Home => "home"
        }
    }
}


pub struct Navigation {
    stack: gtk::Stack,
    browser_factory: BrowserFactory,
    children: RefCell<HashMap<&'static str, Box<dyn Component>>>
}

impl Navigation {

    pub fn new(stack: gtk::Stack, browser_factory: BrowserFactory) -> Self {
        Self { stack, browser_factory, children: RefCell::new(HashMap::new()) }
    }

    fn add_child<W: IsA<gtk::Widget>>(&self, child: &W, name: NavigationPage) {
        child.show_all();
        self.stack.add_named(child, name.into());
    }

    fn add_component(&self, component: Box<dyn Component>, name: NavigationPage) {
        self.children.borrow_mut().insert(name.into(), component);
    }

    fn broadcast(&self, event: AppEvent) {
        for child in self.children.borrow().values() {
            child.on_event(event.clone());
        }
    }

    fn switch_to(&self, name: NavigationPage) {
        self.stack.set_visible_child_name(name.into());
    }

    fn create_browser(&self) {

        let flowbox = gtk::FlowBoxBuilder::new()
            .margin(8)
            .selection_mode(gtk::SelectionMode::None)
            .build();
        let scroll_window = gtk::ScrolledWindowBuilder::new()
            .child(&flowbox)
            .build();

        self.add_child(&scroll_window, NavigationPage::Home);

        let browser: Box<Browser> = Box::new(self.browser_factory.make_browser(flowbox, scroll_window));
        self.add_component(browser, NavigationPage::Home);
    }
}

impl Component for Navigation {

    fn on_event(&self, event: AppEvent) {
        let clone = event.clone();
        match event {
            AppEvent::Started => {
                self.create_browser();
            },
            _ => {}
        };
        self.broadcast(clone);
    }
}
