use gladis::Gladis;
use gtk::prelude::*;
use gtk::ScrolledWindowExt;
use std::rc::Rc;

use crate::app::components::{screen_add_css_provider, Album, Component, EventListener, Playlist};
use crate::app::models::*;
use crate::app::{AppEvent, BrowserEvent, Worker};

use super::UserDetailsModel;

#[derive(Clone, Gladis)]
struct UserDetailsWidget {
    pub root: gtk::ScrolledWindow,
    pub user_name: gtk::Label,
    pub top_artists: gtk::ListBox,
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
    children: Vec<Box<dyn EventListener>>,
}

impl UserDetails {
    pub fn new(model: UserDetailsModel, worker: Worker) -> Self {
        model.load_user_details(model.id.clone());

        let widget = ArtistDetailsWidget::new();
        let model = Rc::new(model);

        let weak_model = Rc::downgrade(&model);
        widget.root.connect_edge_reached(move |_, pos| {
            if let (gtk::PositionType::Bottom, Some(model)) = (pos, weak_model.upgrade()) {
                let _ = model.load_more();
            }
        });

        if let Some(store) = model.get_list_store() {
            let model_clone = Rc::clone(&model);

            widget
                .top_artists
                .bind_model(Some(store.unsafe_store()), move |item| {
                    let item = item.downcast_ref::<AlbumModel>().unwrap();
                    let child = gtk::FlowBoxChild::new();
                    let album = Album::new(item, worker.clone());
                    let weak = Rc::downgrade(&model_clone);
                    album.connect_album_pressed(move |a| {
                        if let (Some(id), Some(m)) = (a.uri().as_ref(), weak.upgrade()) {
                            m.open_album(id);
                        }
                    });
                    child.add(album.get_root_widget());
                    child.show_all();
                    child.upcast::<gtk::Widget>()
                });
        }

        let playlist = Box::new(Playlist::new(widget.top_artists.clone(), Rc::clone(&model)));

        Self {
            widget,
            model,
            children: vec![playlist],
        }
    }

    fn update_details(&self) {
        if let Some(name) = self.model.get_artist_name() {
            let context = self.widget.root.get_style_context();
            context.add_class("user__loaded");
            self.widget.artist_name.set_text(&name);
        }
    }
}

impl Component for UserDetails {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.root.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for ArtistDetails {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::UserDetailsUpdated(id))
                if id == &self.model.id =>
            {
                self.update_details();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
