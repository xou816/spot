use gettextrs::{gettext, ngettext};
use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::{screen_add_css_provider, Component, EventListener};
use crate::app::loader::ImageLoader;
use crate::app::AppEvent;
use crate::app::{BrowserEvent, Worker};

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
        Self {
            worker,
            widget,
            model,
        }
    }

    fn update_info(&mut self) {
        if let Some(info) = self.model.get_album_info() {
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
            self.widget.artists.set_label(&format!(
                "{}: {}",
                ngettext("Artist", "Artists", info.artists.split(", ").count() as u32),
                info.artists
            ));
            self.widget
                .album
                .set_label(&format!("{}: {}", gettext("Album"), info.info.name));
            self.widget
                .id
                .set_label(&format!("{}: {}", gettext("ID"), info.info.id));
            self.widget
                .label
                .set_label(&format!("{}: {}", gettext("Label"), info.info.label));
            self.widget.tracks.set_label(&ngettext(
                "Single",
                format!("Tracks: {}", info.info.total_tracks),
                info.info.total_tracks as u32,
            ));
            self.widget.release_date.set_label(&format!(
                "{}: {}",
                gettext("Release Date"),
                info.info.release_date
            ));
            self.widget.total_time.set_label(&format!(
                "{}: {}",
                gettext("Total Duration"),
                info.formatted_time()
            ));
            self.widget
                .liked
                .set_label(&format!("{}: {}", gettext("Is Liked"), info.is_liked));
            self.widget.markets.set_wrap(true);
            self.widget.copyrights.set_wrap(true);
            self.widget.markets.set_label(&format!(
                "{}: {}",
                ngettext(
                    "Available Market",
                    "Available Markets",
                    info.info.available_markets.len() as u32
                ),
                info.markets()
            ));
            self.widget.copyrights.set_label(&format!(
                "{}: {}",
                ngettext("Copyright", "Copyrights", info.info.copyrights.len() as u32),
                info.copyrights()
            ));
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
