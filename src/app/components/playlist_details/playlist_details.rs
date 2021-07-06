use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use super::PlaylistDetailsModel;

use crate::app::components::{screen_add_css_provider, Component, EventListener, Playlist};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::{AppEvent, BrowserEvent};

#[derive(Gladis, Clone)]
struct PlaylistDetailsWidget {
    pub root: gtk::ScrolledWindow,
    pub name_label: gtk::Label,
    pub owner_button: gtk::LinkButton,
    pub owner_button_label: gtk::Label,
    pub tracks: gtk::ListBox,
    pub art: gtk::Image,
}

impl PlaylistDetailsWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/playlist_details.css"));
        Self::from_resource(resource!("/components/playlist_details.ui")).unwrap()
    }

    fn set_loaded(&self) {
        let context = self.root.style_context();
        context.add_class("playlist_details--loaded");
    }
}

pub struct PlaylistDetails {
    model: Rc<PlaylistDetailsModel>,
    worker: Worker,
    widget: PlaylistDetailsWidget,
    children: Vec<Box<dyn EventListener>>,
}

impl PlaylistDetails {
    pub fn new(model: Rc<PlaylistDetailsModel>, worker: Worker) -> Self {
        if model.get_playlist_info().is_none() {
            model.load_playlist_info();
        }
        let widget = PlaylistDetailsWidget::new();
        let playlist = Box::new(Playlist::new(widget.tracks.clone(), model.clone()));

        widget
            .root
            .connect_edge_reached(clone!(@weak model => move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    model.load_more_tracks();
                }
            }));

        widget.owner_button.connect_activate_link(
            clone!(@weak model => @default-return glib::signal::Inhibit(false), move |_| {
                model.view_owner();
                glib::signal::Inhibit(true)
            }),
        );

        Self {
            model,
            worker,
            widget,
            children: vec![playlist],
        }
    }

    fn update_details(&self) {
        if let Some(info) = self.model.get_playlist_info() {
            let title = &info.title[..];
            let owner = &info.owner.display_name[..];

            self.widget.name_label.set_label(title);
            self.widget.owner_button_label.set_label(owner);

            let widget = self.widget.clone();
            if let Some(art) = info.art.clone() {
                self.worker.send_local_task(async move {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art[..], "jpg", 100, 100)
                        .await;
                    widget.art.set_from_pixbuf(pixbuf.as_ref());
                    widget.set_loaded();
                });
            } else {
                widget.set_loaded();
            }
        }
    }
}

impl Component for PlaylistDetails {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.root.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for PlaylistDetails {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::PlaylistDetailsLoaded(id))
                if id == &self.model.id =>
            {
                self.update_details()
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
