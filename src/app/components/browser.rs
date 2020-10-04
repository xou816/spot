use gtk::prelude::*;
use gtk::ButtonExt;
use gio::prelude::*;

use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::future::Future;

use crate::app::{AppAction, AlbumDescription};
use crate::app::components::{Component};
use super::gtypes::AlbumModel;
use crate::app::dispatch::Worker;
use crate::app::loader::load_remote_image;

const STYLE: &str = "
button.album {
  padding: 0;
  border-radius: 0;
}
";

pub type VecAlbumDescriptionFuture = Pin<Box<dyn Future<Output=Vec<AlbumDescription>>>>;
pub type PlayAlbumFuture = Pin<Box<dyn Future<Output=()>>>;

pub trait BrowserModel {
    fn get_saved_albums(&self) -> VecAlbumDescriptionFuture;
    fn play_album(&self, album_uri: &str) -> PlayAlbumFuture;
}


pub struct Browser {
    browser_model: gio::ListStore,
    model: Rc<dyn BrowserModel>,
    worker: Worker
}

impl Browser {

    pub fn new(flowbox: gtk::FlowBox, worker: Worker, model: Rc<dyn BrowserModel>) -> Self {

        let browser_model = gio::ListStore::new(AlbumModel::static_type());

        let weak_model = Rc::downgrade(&model);
        let worker_clone = worker.clone();
        flowbox.bind_model(Some(&browser_model), move |item| {
            let item = item.downcast_ref::<AlbumModel>().unwrap();
            let child = create_album_for(item, worker_clone.clone(), weak_model.clone());
            child.show_all();
            child.upcast::<gtk::Widget>()
        });

        Self { browser_model, model, worker }
    }

    fn load_saved_albums(&self) {
        let browser_model = self.browser_model.clone();
        let model = Rc::clone(&self.model);
        self.worker.send_task(async move {
            let albums = model.get_saved_albums().await;
            for album in albums {

                let title = glib::markup_escape_text(&album.title);
                let title = format!("<b>{}</b>", title.as_str());

                let artist = glib::markup_escape_text(&album.artist);
                let artist = format!("<small>{}</small>", artist.as_str());

                browser_model.append(&AlbumModel::new(
                    &artist,
                    &title,
                    &album.art,
                    &album.id
                ));
            }
        });
    }
}

impl Component for Browser {
    fn handle(&self, action: &AppAction) {
        match action {
            AppAction::LoginSuccess(_) => {
                self.load_saved_albums();
            },
            _ => {}
        }
    }
}

fn create_album_for(album: &AlbumModel, worker: Worker, model: Weak<dyn BrowserModel>) -> gtk::FlowBoxChild {
    let child = gtk::FlowBoxChild::new();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    child.add(&vbox);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox.set_halign(gtk::Align::Center);

    {
        let album = album.clone();

        let image = gtk::Image::new();
        let image_clone = image.clone();
        if let Some(url) = album.cover_url() {
            worker.send_task(async move {
                let result = load_remote_image(&url, 120, 120).await;
                image_clone.set_from_pixbuf(result.as_ref());
            });
        }

        let button = gtk::Button::new();
        button.set_relief(gtk::ReliefStyle::None);
        button.set_margin_top(0);
        button.connect_clicked(move |_| {
            if let (Some(model), Some(uri)) = (model.upgrade(), album.uri()) {
                worker.send_task(model.play_album(&uri));
            }
        });

        let provider = gtk::CssProvider::new();
        provider
            .load_from_data(STYLE.as_bytes())
            .expect("Failed to load CSS");
        let context = button.get_style_context();
        context.add_class("album");
        context.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        button.add(&image);
        hbox.add(&button)
    }

    vbox.pack_start(&hbox, false, false, 6);

    let label = gtk::Label::new(None);
    label.set_use_markup(true);
    label.set_halign(gtk::Align::Center);
    label.set_line_wrap(true);
    label.set_max_width_chars(25);

    album.bind_property("album", &label, "label")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

    vbox.pack_start(&label, false, false, 3);

    let label = gtk::Label::new(None);
    label.set_use_markup(true);
    label.set_halign(gtk::Align::Center);
    label.set_line_wrap(true);
    label.set_max_width_chars(25);

    album.bind_property("artist", &label, "label")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

    vbox.pack_start(&label, false, false, 3);

    child
}
