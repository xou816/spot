use std::rc::Rc;
use std::cell::{Ref, RefCell};
use crate::app::{BrowserAction, ActionDispatcher, AppModel};
use crate::app::state::ScreenName;
use super::NavigationModel;


pub struct NavigationModelImpl {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl NavigationModelImpl {

    pub fn new(app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self { app_model, dispatcher }
    }
}

impl NavigationModel for NavigationModelImpl {

    fn go_back(&self) {
        self.dispatcher.dispatch(BrowserAction::NavigationPop.into())
    }

    fn visible_child_name(&self) -> Ref<'_, ScreenName> {
        Ref::map(self.app_model.borrow(), |m| m.state.browser_state.current_screen())
    }

    fn can_go_back(&self) -> bool {
        self.app_model.borrow().state.browser_state.can_pop()
    }

    fn children_count(&self) -> usize {
        self.app_model.borrow().state.browser_state.count()
    }
}
