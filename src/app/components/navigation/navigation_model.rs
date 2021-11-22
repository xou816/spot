use crate::app::state::ScreenName;
use crate::app::{ActionDispatcher, AppModel, BrowserAction};
use std::ops::Deref;
use std::rc::Rc;

pub struct NavigationModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl NavigationModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn go_home(&self) {
        self.dispatcher
            .dispatch(BrowserAction::NavigationPopTo(ScreenName::Home).into())
    }

    pub fn visible_child_name(&self) -> impl Deref<Target = ScreenName> + '_ {
        self.app_model.map_state(|s| s.browser.current_screen())
    }

    pub fn set_nav_hidden(&self, hidden: bool) {
        self.dispatcher
            .dispatch(BrowserAction::SetNavigationHidden(hidden).into());
    }

    pub fn children_count(&self) -> usize {
        self.app_model.get_state().browser.count()
    }
}
