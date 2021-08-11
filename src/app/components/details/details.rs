use gettextrs::{gettext, ngettext};
use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use super::DetailsModel;

use crate::app::components::{screen_add_css_provider, Component, EventListener, Playlist};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::{AppEvent, BrowserEvent};

#[derive(Gladis, Clone)]
struct DetailsWidget {
    pub root: gtk::Widget,
    pub album_label: gtk::Label,
    pub album_tracks: gtk::ListBox,
    pub album_art: gtk::Image,
    pub like_button: gtk::Button,
    pub artist_button: gtk::LinkButton,
    pub artist_button_label: gtk::Label,
    pub album_info: gtk::Button,
    info_window: libhandy::Window,
    info_close: gtk::Button,
    info_art: gtk::Image,
    info_album_artist: gtk::Label,
    info_label: gtk::Label,
    info_release: gtk::Label,
    info_tracks: gtk::Label,
    info_duration: gtk::Label,
    info_copyright: gtk::Label,
}

impl DetailsWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/details.css"));
        Self::from_resource(resource!("/components/details.ui")).unwrap()
    }

    fn set_loaded(&self) {
        let context = self.root.style_context();
        context.add_class("details--loaded");
    }
}

pub struct Details {
    model: Rc<DetailsModel>,
    worker: Worker,
    widget: DetailsWidget,
    children: Vec<Box<dyn EventListener>>,
}

impl Details {
    pub fn new(model: Rc<DetailsModel>, worker: Worker) -> Self {
        if model.get_album_info().is_none() {
            model.load_album_info();
        }
        let widget = DetailsWidget::new();
        let playlist = Box::new(Playlist::new(widget.album_tracks.clone(), model.clone()));

        widget
            .like_button
            .connect_clicked(clone!(@weak model => move |_| {
                model.toggle_save_album();
            }));

        let info_window = widget.info_window.clone();
        info_window.connect_delete_event(|info, _| {
            info.hide();
            glib::signal::Inhibit(true)
        });
        info_window.connect_key_press_event(|info, event| {
            if let gdk::keys::constants::Escape = event.keyval() {
                info.hide()
            }
            glib::signal::Inhibit(false)
        });
        widget
            .album_info
            .connect_clicked(move |_| info_window.show());

        let info = widget.info_window.clone();
        widget.info_close.connect_clicked(move |_| info.hide());

        Self {
            model,
            worker,
            widget,
            children: vec![playlist],
        }
    }

    fn update_liked(&self) {
        if let Some(info) = self.model.get_album_info() {
            let is_liked = info.is_liked;
            self.widget
                .like_button
                .set_label(if is_liked { "♥" } else { "♡" });
        }
    }

    fn update_details(&mut self) {
        if let Some(info) = self.model.get_album_info() {
            let album = &info.title[..];
            let artist = &info.artists_name();

            self.widget.album_label.set_label(album);
            self.widget.artist_button_label.set_label(artist);
            let weak_model = Rc::downgrade(&self.model);
            self.widget.artist_button.connect_activate_link(move |_| {
                if let Some(model) = weak_model.upgrade() {
                    model.view_artist();
                }
                glib::signal::Inhibit(true)
            });

            let widget = self.widget.clone();
            if let Some(art) = info.art.clone() {
                self.worker.send_local_task(async move {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art[..], "jpg", 100, 100)
                        .await;
                    widget.album_art.set_from_pixbuf(pixbuf.as_ref());
                    widget.set_loaded();
                });
            } else {
                widget.set_loaded();
            }
        }
    }

    fn update_dialog(&mut self) {
        if let Some(album) = self.model.get_album_info() {
            let widget = self.widget.clone();
            if let Some(art) = album.art.clone() {
                self.worker.send_local_task(async move {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art[..], "jpg", 200, 200)
                        .await;
                    widget.info_art.set_from_pixbuf(pixbuf.as_ref());
                    widget.set_loaded();
                });
            } else {
                widget.set_loaded();
            }
            self.widget.info_album_artist.set_text(&format!(
                "{} {} {}",
                album.title,
                gettext("by"),
                album.artists_name()
            ));

            self.widget
                .info_label
                .set_text(&format!("{}: {}", gettext("Label"), album.label));

            self.widget.info_release.set_text(&format!(
                "{}: {}",
                gettext("Released"),
                album.release_date
            ));

            self.widget.info_tracks.set_text(&format!(
                "{}: {}",
                gettext("Tracks"),
                album.songs.len()
            ));

            self.widget.info_duration.set_text(&format!(
                "{}: {}",
                gettext("Duration"),
                album.formatted_time()
            ));

            self.widget.info_copyright.set_text(&format!(
                "{}: {}",
                ngettext("Copyright", "Copyrights", album.copyrights.len() as u32),
                album.copyrights()
            ));
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
            AppEvent::BrowserEvent(BrowserEvent::AlbumDetailsLoaded(id))
                if id == &self.model.id =>
            {
                self.update_details();
                self.update_liked();
                self.update_dialog();
            }
            AppEvent::BrowserEvent(BrowserEvent::AlbumSaved(id))
            | AppEvent::BrowserEvent(BrowserEvent::AlbumUnsaved(id))
                if id == &self.model.id =>
            {
                self.update_liked();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
