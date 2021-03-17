use gladis::Gladis;
use gtk::prelude::*;
use gtk::ScrolledWindowExt;
use std::rc::{Rc, Weak};

use crate::app::components::{
    screen_add_css_provider, utils::wrap_flowbox_item, Album, Component, EventListener,
};
use crate::app::models::*;
use crate::app::{AppEvent, BrowserEvent, Worker};

use super::UserDetailsModel;

#[derive(Clone, Gladis)]
struct UserDetailsWidget {
    pub root: gtk::ScrolledWindow,
    pub user_name: gtk::Label,
    pub user_playlists: gtk::FlowBox,
}

impl UserDetailsWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/user_details.css"));
        Self::from_resource(resource!("/components/user_details.ui")).unwrap()
    }
}

pub struct UserDetails {
    model: Rc<UserDetailsModel>,
    widget: UserDetailsWidget,
}

impl UserDetails {
    pub fn new(model: UserDetailsModel, worker: Worker) -> Self {
        model.load_user_details(model.id.clone());

        let widget = UserDetailsWidget::new();
        let model = Rc::new(model);

        widget
            .root
            .connect_edge_reached(clone!(@weak model => move |_, pos| {
                if pos == gtk::PositionType::Bottom {
                    let _ = model.load_more();
                }
            }));

        if let Some(store) = model.get_list_store() {
            let weak_model = Rc::downgrade(&model);

            widget
                .user_playlists
                .bind_model(Some(store.unsafe_store()), move |item| {
                    wrap_flowbox_item(item, |item: &AlbumModel| {
                        let album = Album::new(item, worker.clone());
                        let weak = weak_model.clone();
                        album.connect_album_pressed(move |a| {
                            if let (Some(id), Some(model)) = (a.uri().as_ref(), weak.upgrade()) {
                                model.open_playlist(id);
                            }
                        });
                        album.get_root_widget().clone()
                    })
                });
        }

        Self { widget, model }
    }

    fn update_details(&self) {
        if let Some(name) = self.model.get_user_name() {
            let context = self.widget.root.get_style_context();
            context.add_class("user__loaded");
            self.widget.user_name.set_text(&name);
        }
    }
}

impl Component for UserDetails {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.root.upcast_ref()
    }
}

impl EventListener for UserDetails {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::UserDetailsUpdated(id))
                if id == &self.model.id =>
            {
                self.update_details();
            }
            _ => {}
        }
    }
}
