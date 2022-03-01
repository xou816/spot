use crate::app::state::UpdatableState;
use crate::app::AppEvent;
use crate::AppAction;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum SidebarAction {
    SelectItem(String, i32),
}

impl From<SidebarAction> for AppAction {
    fn from(sidebar_action: SidebarAction) -> Self {
        Self::SidebarAction(sidebar_action)
    }
}

#[derive(Clone, Debug)]
pub enum SidebarEvent {
    ItemSelected(String),
}

impl From<SidebarEvent> for AppEvent {
    fn from(sidebar_event: SidebarEvent) -> Self {
        Self::SidebarEvent(sidebar_event)
    }
}

pub struct SidebarState {
    currently_selected_item: String,
    previously_selected_item: String,
    currently_selected_id: i32,
    previously_selected_id: i32,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            currently_selected_item: Default::default(),
            previously_selected_item: Default::default(),
            currently_selected_id: Default::default(),
            previously_selected_id: Default::default(),
        }
    }
}

impl SidebarState {
    pub fn get_previously_selected_item(&self) -> &String {
        &self.previously_selected_item
    }
    pub fn get_currently_selected_item(&self) -> &String {
        &self.currently_selected_item
    }

    pub fn get_previously_selected_index(&self) -> &i32 {
        &self.previously_selected_id
    }

    pub fn get_currently_selected_index(&self) -> &i32 {
        &self.currently_selected_id
    }
}

impl UpdatableState for SidebarState {
    type Action = SidebarAction;
    type Event = SidebarEvent;

    fn update_with(&mut self, action: Cow<Self::Action>) -> Vec<Self::Event> {
        match action.into_owned() {
            SidebarAction::SelectItem(item, id) => {
                self.previously_selected_item = self.currently_selected_item.clone();
                self.previously_selected_id = self.currently_selected_id.clone();
                self.currently_selected_id = id.clone();
                self.currently_selected_item = item.clone();
                vec![SidebarEvent::ItemSelected(item)]
            }
        }
    }
}
