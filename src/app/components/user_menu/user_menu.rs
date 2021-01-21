use gio::{ActionMapExt, SimpleAction, SimpleActionGroup};
use gtk::{MenuButtonExt, WidgetExt};
use std::rc::Rc;

use super::UserMenuModel;
use crate::app::components::EventListener;
use crate::app::AppEvent;

pub struct UserMenu {
    user_button: gtk::MenuButton,
    model: Rc<UserMenuModel>,
}

impl UserMenu {
    pub fn new(user_button: gtk::MenuButton, model: UserMenuModel) -> Self {
        let model = Rc::new(model);

        Self { user_button, model }
    }

    fn update_menu(&self) {
        if let Some(username) = self.model.username() {
            let action_group = SimpleActionGroup::new();
            let logout = SimpleAction::new("logout", None);
            let weak_model = Rc::downgrade(&self.model);
            logout.connect_activate(move |_, _| {
                if let Some(model) = weak_model.upgrade() {
                    model.logout();
                }
            });
            action_group.add_action(&logout);

            let menu = gio::Menu::new();
            let user_menu = gio::Menu::new();
            user_menu.insert(0, Some("Log out"), Some("user.logout"));
            menu.insert_section(0, Some(&username), &user_menu);

            self.user_button
                .insert_action_group("user", Some(&action_group));
            self.user_button.set_menu_model(Some(&menu));
        }
    }
}

impl EventListener for UserMenu {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::LoginCompleted(_) => {
                self.update_menu();
            }
            _ => {}
        }
    }
}
