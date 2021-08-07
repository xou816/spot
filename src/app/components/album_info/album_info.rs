use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::{BrowserEvent, Worker};
use crate::app::components::{screen_add_css_provider, Component, EventListener};
use crate::app::AppEvent;
use crate::app::loader::ImageLoader;

use super::AlbumInfoModel;

#[derive(Gladis, Clone)]
struct AlbumInfoWidget {
    pub root: gtk::Widget,
    pub album_art: gtk::Image,
    pub artists: gtk::Label,
    pub album: gtk::Label,
    pub id: gtk::Label,
    pub label: gtk::Label,
    pub tracks: gtk::Label,
    pub release_date: gtk::Label,
    pub total_time: gtk::Label,
    pub liked: gtk::Label,
    pub copyrights: gtk::Label,
    pub markets: gtk::Label,
}

impl AlbumInfoWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/album_info.css"));
        Self::from_resource(resource!("/components/album_info.ui")).unwrap()
    }

    fn set_loaded(&self) {
        let context = self.root.style_context();
        context.add_class("details--loaded");
    }
}

pub struct Info {
    worker: Worker,
    widget: AlbumInfoWidget,
    model: Rc<AlbumInfoModel>,
}

impl Info {
    pub fn new(model: Rc<AlbumInfoModel>, worker: Worker) -> Self {
        let widget = AlbumInfoWidget::new();
        model.load_album_info_detail();
        Self { worker, widget, model }
    }

    fn update_info(&mut self) {
        if let Some(info) = self.model.get_album_info() {
            self.widget.artists.set_label(&format!("Artists: {}", info.artists));
            self.widget.album.set_label(&format!("Album: {}" , info.info.name));
            self.widget.id.set_label(&format!("ID: {}" , info.info.id));
            self.widget.label.set_label(&format!("Label: {}" , info.info.label));
            self.widget.tracks.set_label(&format!("Tracks: {}" , info.info.total_tracks));
            self.widget.release_date.set_label(&format!("Release Date: {}" , info.info.release_date));
            self.widget.total_time.set_label(&format!("Total Duration: {}" , info.formatted_time()));
            self.widget.liked.set_label(&format!("Is Liked: {}" ,info.is_liked));
            // TODO do copyrights and markets (markets needs some wrapping)
            //self.widget.markets.set_label(&info.markets());
            let widget = self.widget.clone();
            if let Some(art) = info.art.clone() {
                self.worker.send_local_task(async move {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art[..], "jpg", 200, 200)
                        .await;
                    widget.album_art.set_from_pixbuf(pixbuf.as_ref());
                    widget.set_loaded();
                });
            } else {
                widget.set_loaded();
            }
        }
    }
}

impl Component for Info {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        None
    }
}

impl EventListener for Info {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::BrowserEvent(BrowserEvent::AlbumInfoUpdated) = event {
            self.update_info();
        }
        self.broadcast_event(event);
    }
}