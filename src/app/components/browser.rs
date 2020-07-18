use gtk::prelude::*;
use gtk::ButtonExt;
use gio::prelude::*;

use std::rc::{Rc, Weak};
use std::cell::RefCell;

use crate::app::{AppAction, AlbumDescription};
use crate::app::components::{Component};
use super::gtypes::AlbumModel;
use crate::app::dispatch::Worker;
use crate::app::loader::load_remote_image;

pub trait BrowserModel {
    fn get_saved_albums(&self, completion: Box<dyn Fn(Vec<AlbumDescription>) -> ()>);
    fn play_album(&self, album_uri: &str);
}


pub struct Browser {
    browser_model: gio::ListStore,
    model: Rc<RefCell<dyn BrowserModel>>
}

impl Browser {

    pub fn new(builder: &gtk::Builder, worker: Worker, model: Rc<RefCell<dyn BrowserModel>>) -> Self {

        let browser_model = gio::ListStore::new(AlbumModel::static_type());
        let flowbox: gtk::FlowBox = builder.get_object("flowbox").unwrap();

        let weak_model = Rc::downgrade(&model);
        flowbox.bind_model(Some(&browser_model), move |item| {
            let item = item.downcast_ref::<AlbumModel>().unwrap();
            let child = create_album_for(item, worker.clone(), weak_model.clone());
            child.show_all();
            child.upcast::<gtk::Widget>()
        });

        Self { browser_model, model }
    }

    fn load_saved_albums(&self) {
        let browser_model = self.browser_model.clone();
        let model = self.model.borrow();

        model.get_saved_albums(Box::new(move |albums| {
            for album in albums {
                browser_model.append(&AlbumModel::new(
                    &album.artist,
                    &album.title,
                    &album.art,
                    &album.id
                ));
            }
        }));
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

fn create_album_for(album: &AlbumModel, worker: Worker, model: Weak<RefCell<BrowserModel>>) -> gtk::FlowBoxChild {
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
                model.borrow().play_album(&uri);
            }
        });

        button.add(&image);
        hbox.add(&button)
    }

    vbox.pack_start(&hbox, false, false, 6);

    let label = gtk::Label::new(None);
    label.set_use_markup(true);
    label.set_halign(gtk::Align::Center);

    album.bind_property("artist", &label, "label")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

    vbox.pack_start(&label, false, false, 6);

    child
}
