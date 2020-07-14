use gtk::prelude::*;
use gtk::OverlayExt;
use gio::prelude::*;

use crate::app::backend::api;
use crate::app::{AppAction};
use crate::app::components::{Component};
use super::gtypes::AlbumModel;
use crate::app::dispatch::Worker;
use crate::app::loader::load_remote_image;


pub struct Browser {
    browser_model: gio::ListStore,
    worker: Worker
}

impl Browser {

    pub fn new(builder: &gtk::Builder, worker: Worker) -> Self {

        let browser_model = gio::ListStore::new(AlbumModel::static_type());
        let flowbox: gtk::FlowBox = builder.get_object("flowbox").unwrap();

        let worker_clone = worker.clone();

        flowbox.bind_model(Some(&browser_model), move |item| {
            let item = item.downcast_ref::<AlbumModel>().unwrap();
            let child = create_album_for(item, worker.clone());
            child.show_all();
            child.upcast::<gtk::Widget>()
        });

        Self { browser_model, worker: worker_clone }
    }

    fn load_saved_albums(&self, token: String) {
        let browser_model = self.browser_model.clone();
        self.worker.send_task(async move {
            if let Some(albums) = api::get_saved_albums(token).await {
                for album in albums {
                    browser_model.append(&AlbumModel::new(
                        &album.artist,
                        &album.title,
                        &album.art,
                        &album.uri
                    ));
                }
            }
        });
    }
}

impl Component for Browser {
    fn handle(&self, action: &AppAction) {
        match action {
            AppAction::LoginSuccess(creds) => {
                self.load_saved_albums(creds.token.clone());
            },
            _ => {}
        }
    }
}

fn create_album_for(album: &AlbumModel, worker: Worker) -> gtk::FlowBoxChild {
    let child = gtk::FlowBoxChild::new();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    child.add(&vbox);

    let image = gtk::Image::new();
    let image_clone = image.clone();
    if let Some(url) = album.cover_url() {
        worker.send_task(async move {
            let result = load_remote_image(&url, 120, 120).await;
            image_clone.set_from_pixbuf(result.as_ref());
        });
    }

    vbox.pack_start(&image, false, false, 6);

    let label = gtk::Label::new(None);
    label.set_use_markup(true);
    label.set_halign(gtk::Align::Center);

    album.bind_property("artist", &label, "label")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

    vbox.pack_start(&label, false, false, 6);

    child
}
