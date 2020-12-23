use std::rc::Rc;
use gtk::prelude::*;
use gtk::LinkButtonExt;
use gladis::Gladis;

use crate::app::components::{Component, EventListener, screen_add_css_provider, PlaylistFactory};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::{AppEvent, BrowserEvent};
use super::DetailsModel;

#[derive(Gladis, Clone)]
struct DetailsWidget {
    pub root: gtk::Widget,
    pub album_label: gtk::Label,
    pub album_tracks: gtk::ListBox,
    pub album_art: gtk::Image,
    pub like_button: gtk::Button,
    pub artist_button: gtk::LinkButton
}

impl DetailsWidget {

    fn new() -> Self {
        screen_add_css_provider(resource!("/components/details.css"));
        Self::from_resource(resource!("/components/details.ui")).unwrap()
    }

    fn set_loaded(&self) {
        let context = self.root.get_style_context();
        context.add_class("details--loaded");
    }
}

pub struct Details {
    model: Rc<DetailsModel>,
    worker: Worker,
    widget: DetailsWidget,
    children: Vec<Box<dyn EventListener>>
}

impl Details {

    pub fn new(id: String, model: DetailsModel, worker: Worker, playlist_factory: &PlaylistFactory) -> Self {

        if model.get_album_info().is_none() {
            model.load_album_info(id);
        }

        let model = Rc::new(model);
        let widget = DetailsWidget::new();
        let playlist = Box::new(playlist_factory.make_custom_playlist(widget.album_tracks.clone()));

        Self { model, worker, widget, children: vec![playlist] }
    }

    fn update_details(&self) {
        if let Some(info) = self.model.get_album_info() {

            let album = &info.title[..];
            let artist = &info.artist[..];
            let art = info.art.clone();
            let is_liked = false;

            self.widget.album_label.set_label(album);
            self.widget.like_button.set_label(if is_liked { "♥" } else { "♡" });
            self.widget.artist_button.set_label(artist);


            let weak_model = Rc::downgrade(&self.model);
            self.widget.artist_button.connect_activate_link(move |_| {
                if let Some(model) = weak_model.upgrade() {
                    model.view_artist();
                }
                glib::signal::Inhibit(true)
            });

            let widget = self.widget.clone();
            self.worker.send_local_task(async move {
                let pixbuf = ImageLoader::new()
                    .load_remote(&art[..], "jpg", 100, 100).await;
                widget.album_art.set_from_pixbuf(pixbuf.as_ref());
                widget.set_loaded();
            });
        }
    }
}

impl Component for Details {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}


impl EventListener for Details {

    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::DetailsLoaded) => self.update_details(),
            _ => {}
        }
        self.broadcast_event(event);
    }
}
