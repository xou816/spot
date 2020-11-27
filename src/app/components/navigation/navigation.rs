use gtk::prelude::*;
use gtk::{ButtonExt, ContainerExt};
use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::app::components::{EventListener, ListenerComponent, DetailsFactory, BrowserFactory, SearchFactory};
use crate::app::{AppEvent, BrowserEvent};
use crate::app::state::ScreenName;

pub trait NavigationModel {
    fn go_back(&self);
    fn can_go_back(&self) -> bool;
    fn visible_child_name(&self) -> Ref<'_, ScreenName>;
    fn children_count(&self) -> usize;
}

pub struct Navigation {
    model: Rc<dyn NavigationModel>,
    stack: gtk::Stack,
    back_button: gtk::Button,
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

        Self { model, stack, back_button, browser_factory, details_factory, search_factory, children: RefCell::new(vec![]) }
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
        self.children.borrow_mut().push(component);
        self.stack.set_visible_child_name(name.identifier());
    }


    fn pop(&self) {
        let mut children = self.children.borrow_mut();
        let popped = children.pop();

        let name = self.model.visible_child_name();
        self.stack.set_visible_child_name(name.identifier());

        if let Some(child) = popped {
            self.stack.remove(child.get_root_widget());
        }
    }

    fn pop_to(&self, screen: &ScreenName) {
        self.stack.set_visible_child_name(screen.identifier());
        let remainder = self.children.borrow_mut().split_off(self.model.children_count());
        for widget in remainder {
            self.stack.remove(widget.get_root_widget());
        }
    }

    fn update_back_button(&self) {
        self.back_button.set_sensitive(self.model.can_go_back());
    }
}

impl EventListener for Navigation {

    fn on_event(&self, event: &AppEvent) {
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
        if let Some(listener) = self.children.borrow().iter().last() {
            listener.on_event(event);
        }
    }
}
