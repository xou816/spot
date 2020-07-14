use gtk::prelude::*;
use gtk::OverlayExt;
use gio::prelude::*;

use crate::app::{AppAction};
use crate::app::components::{Component};
use super::gtypes::AlbumModel;
use crate::app::dispatch::Worker;
use crate::app::loader::load_remote_image;


pub struct Browser {}

impl Browser {

    pub fn new(builder: &gtk::Builder, worker: Worker) -> Self {

        let browser_model = gio::ListStore::new(AlbumModel::static_type());
        let flowbox: gtk::FlowBox = builder.get_object("flowbox").unwrap();

        browser_model.append(&AlbumModel::new(
            "The Velvet Underground and Nico",
            "The Velvet Underground and Nico",
            "https://images-na.ssl-images-amazon.com/images/I/71YJlc9Wb6L._SL1500_.jpg",
            "spotify:uri:foobar"
        ));

                browser_model.append(&AlbumModel::new(
            "The Velvet Underground and Nico",
            "The Velvet Underground and Nico",
            "https://images-na.ssl-images-amazon.com/images/I/71YJlc9Wb6L._SL1500_.jpg",
            "spotify:uri:foobar"
        ));

                browser_model.append(&AlbumModel::new(
            "The Velvet Underground and Nico",
            "The Velvet Underground and Nico",
            "https://images-na.ssl-images-amazon.com/images/I/71YJlc9Wb6L._SL1500_.jpg",
            "spotify:uri:foobar"
        ));

                browser_model.append(&AlbumModel::new(
            "The Velvet Underground and Nico",
            "The Velvet Underground and Nico",
            "https://images-na.ssl-images-amazon.com/images/I/71YJlc9Wb6L._SL1500_.jpg",
            "spotify:uri:foobar"
        ));

                browser_model.append(&AlbumModel::new(
            "The Velvet Underground and Nico",
            "The Velvet Underground and Nico",
            "https://images-na.ssl-images-amazon.com/images/I/71YJlc9Wb6L._SL1500_.jpg",
            "spotify:uri:foobar"
        ));

                browser_model.append(&AlbumModel::new(
            "The Velvet Underground and Nico",
            "The Velvet Underground and Nico",
            "https://images-na.ssl-images-amazon.com/images/I/71YJlc9Wb6L._SL1500_.jpg",
            "spotify:uri:foobar"
        ));

        flowbox.bind_model(Some(&browser_model), move |item| {
            let item = item.downcast_ref::<AlbumModel>().unwrap();
            let child = create_album_for(item, worker.clone());
            child.show_all();
            child.upcast::<gtk::Widget>()
        });

        Self {}
    }
}

impl Component for Browser {
    fn handle(&self, action: &AppAction) {}
}

fn create_album_for(album: &AlbumModel, worker: Worker) -> gtk::FlowBoxChild {
    let child = gtk::FlowBoxChild::new();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    child.add(&vbox);

    let overlay = gtk::Overlay::new();

    let image = gtk::Image::new();
    let image_clone = image.clone();
    if let Some(url) = album.cover_url() {
        worker.send_task(async move {
            let result = load_remote_image(&url, 120, 120).await;
            image_clone.set_from_pixbuf(result.as_ref());
        });
    }

    overlay.add(&image);

    let _box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let button = gtk::Button::new();
    let image = gtk::Image::new_from_icon_name(Some("media-playback-start"), gtk::IconSize::Button);
    button.add(&image);
    button.set_relief(gtk::ReliefStyle::None);
    button.set_vexpand(false);
    button.set_hexpand(false);
    _box.add(&button);

    overlay.add_overlay(&_box);

    vbox.pack_start(&overlay, false, false, 6);

    let label = gtk::Label::new(None);
    label.set_use_markup(true);
    label.set_halign(gtk::Align::Center);

    album.bind_property("artist", &label, "label")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

    vbox.pack_start(&label, false, false, 6);

    child
}
