use gladis::Gladis;
use gtk::prelude::*;
use gtk::ScrolledWindowExt;
use std::rc::{Rc, Weak};

use crate::app::components::{screen_add_css_provider, Album, Component, EventListener};
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
            let worker_clone = worker.clone();

            widget
                .user_playlists
                .bind_model(Some(store.unsafe_store()), move |item| {
                    let item = item.downcast_ref::<AlbumModel>().unwrap();
                    let child = create_album_for(item, worker_clone.clone(), weak_model.clone());
                    child.show_all();
                    child.upcast::<gtk::Widget>()
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

fn create_album_for(
    album_model: &AlbumModel,
    worker: Worker,
    model: Weak<UserDetailsModel>,
) -> gtk::FlowBoxChild {
    let child = gtk::FlowBoxChild::new();

    let album = Album::new(album_model, worker);
    child.add(album.get_root_widget());

    album.connect_album_pressed(move |a| {
        if let (Some(model), Some(id)) = (model.upgrade(), a.uri()) {
            model.open_playlist(&id);
        }
    });

    child
}
