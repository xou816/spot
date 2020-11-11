use gtk::prelude::*;
use gtk::ScrolledWindowExt;
use gio::prelude::*;

use std::iter::Iterator;
use std::rc::{Rc, Weak};
use std::cell::Ref;

use crate::app::{AppEvent, AlbumDescription, BrowserEvent};
use crate::app::components::{Component, EventListener, gtypes::AlbumModel, Album};
use crate::app::dispatch::Worker;

pub trait BrowserModel {
    fn get_saved_albums(&self) -> Option<Ref<'_, Vec<AlbumDescription>>>;
    fn refresh_saved_albums(&self);
    fn load_more_albums(&self);
    fn play_album(&self, album_uri: &str);
    fn open_album(&self, album_uri: &str);
}


pub struct Browser {
    root: gtk::Widget,
    browser_model: gio::ListStore,
    model: Rc<dyn BrowserModel>
}

impl Browser {

    pub fn new(worker: Worker, model: Rc<dyn BrowserModel>) -> Self {

        let flowbox = gtk::FlowBoxBuilder::new()
            .margin(8)
            .selection_mode(gtk::SelectionMode::None)
            .build();

        let scroll_window = gtk::ScrolledWindowBuilder::new()
            .child(&flowbox)
            .build();

        let browser_model = gio::ListStore::new(AlbumModel::static_type());

        let weak_model = Rc::downgrade(&model);
        let worker_clone = worker.clone();
        flowbox.bind_model(Some(&browser_model), move |item| {
            let item = item.downcast_ref::<AlbumModel>().unwrap();
            let child = create_album_for(item, worker_clone.clone(), weak_model.clone());
            child.show_all();
            child.upcast::<gtk::Widget>()
        });

        let weak_model = Rc::downgrade(&model);
        scroll_window.connect_edge_reached(move |_, pos| {
            if let (gtk::PositionType::Bottom, Some(model)) = (pos, weak_model.upgrade()) {
                model.load_more_albums();
            }
        });

        Self { root: scroll_window.upcast(), browser_model, model }
    }

    fn set_saved_albums(&self) {
        self.browser_model.remove_all();
        if let Some(albums) = self.model.get_saved_albums() {
            self.append_albums(albums.iter());
        }
    }

    fn append_next_albums(&self, offset: usize) {
        if let Some(albums) = self.model.get_saved_albums() {
            self.append_albums(albums.iter().skip(offset));
        }
    }

    fn append_albums<'a>(&self, albums: impl Iterator<Item=&'a AlbumDescription>) {

        for album in albums {
            self.browser_model.append(&AlbumModel::new(
                &album.artist,
                &album.title,
                &album.art,
                &album.id
            ));
        }
    }
}

impl EventListener for Browser {

    fn on_event(&self, event: &AppEvent) {
        match event {
            AppEvent::Started|AppEvent::LoginCompleted => {
                self.model.refresh_saved_albums();
            },
            AppEvent::BrowserEvent(BrowserEvent::ContentSet) => {
                self.set_saved_albums();
            },
            AppEvent::BrowserEvent(BrowserEvent::ContentAppended(offset)) => {
                self.append_next_albums(*offset);
            }
            _ => {}
        }
    }
}

impl Component for Browser {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.root
    }
}

fn create_album_for(album_model: &AlbumModel, worker: Worker, model: Weak<dyn BrowserModel>) -> gtk::FlowBoxChild {
    let child = gtk::FlowBoxChild::new();

    let album = Album::new(album_model, worker);
    child.add(album.get_root_widget());

    album.connect_album_pressed(move |a| {
        if let (Some(model), Some(uri)) = (model.upgrade(), a.uri()) {
            model.open_album(&uri);
        }
    });

    child
}
