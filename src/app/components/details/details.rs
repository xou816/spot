use std::rc::Rc;
use std::cell::Ref;
use gtk::prelude::*;
use gladis::Gladis;

use crate::app::components::{Component, EventListener, screen_add_css_provider, PlaylistFactory};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::{AppEvent, BrowserEvent};
use crate::app::models::*;

pub trait DetailsModel {
    fn get_album_info(&self) -> Option<Ref<'_, AlbumDescription>>;
}

#[derive(Gladis, Clone)]
struct DetailsWidget {
    pub root: gtk::Widget,
    pub artist_label: gtk::Label,
    pub album_label: gtk::Label,
    pub album_tracks: gtk::ListBox,
    pub album_art: gtk::Image,
    pub like_button: gtk::Button
}

impl DetailsWidget {

    fn new() -> Self {
        screen_add_css_provider(resource!("/components/details.css"));
        Self::from_resource(resource!("/components/details.ui")).unwrap()
    }
}

pub struct Details {
    model: Rc<dyn DetailsModel>,
    worker: Worker,
    widget: DetailsWidget,
    children: Vec<Box<dyn EventListener>>
}

impl Details {

    pub fn new(model: Rc<dyn DetailsModel>, worker: Worker, playlist_factory: &PlaylistFactory) -> Self {

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
            self.widget.artist_label.set_label(artist);
            self.widget.like_button.set_label(if is_liked { "♥" } else { "♡" });

            let image = self.widget.album_art.clone();
            self.worker.send_task(async move {
                let pixbuf = ImageLoader::new()
                    .load_remote(&art[..], "jpg", 100, 100).await;
                image.set_from_pixbuf(pixbuf.as_ref());
            });
        }
    }
}

impl Component for Details {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }

    fn get_children(&self) -> Option<&Vec<Box<dyn EventListener>>> {
        Some(&self.children)
    }
}


impl EventListener for Details {

    fn on_event(&self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::DetailsLoaded) => self.update_details(),
            _ => {}
        }
        self.broadcast_event(event);
    }
}
