use std::rc::Rc;
use std::cell::Ref;
use gtk::prelude::*;
use super::{Component, EventListener};
use crate::app::connect::PlaylistFactory;
use crate::app::{AppEvent, BrowserEvent};
use crate::app::models::*;

pub trait DetailsModel {
    fn get_album_info(&self) -> Option<Ref<'_, AlbumDescription>>;
}

pub struct Details {
    model: Rc<dyn DetailsModel>,
    album_name: gtk::Label,
    root: gtk::Widget,
    children: Vec<Box<dyn EventListener>>
}

impl Details {

    pub fn new(model: Rc<dyn DetailsModel>, playlist_factory: &PlaylistFactory) -> Self {

        let album_name = gtk::LabelBuilder::new()
            .label("Loading")
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        let context = album_name.get_style_context();
        context.add_class("title");

        let listbox = gtk::ListBox::new();

        let _box = gtk::BoxBuilder::new()
            .margin(16)
            .orientation(gtk::Orientation::Vertical)
            .build();

        _box.pack_start(&album_name, false, false, 8);
        _box.pack_start(&listbox, false, false, 8);

        let root = gtk::ScrolledWindowBuilder::new()
            .child(&_box)
            .build()
            .upcast::<gtk::Widget>();

        let playlist = Box::new(playlist_factory.make_custom_playlist(listbox.clone()));

        Self { model, album_name, root, children: vec![playlist] }
    }

    fn update_details(&self) {
        let info = self.model.get_album_info();

        let album_title = info.as_ref().map(|i| &i.title[..]).unwrap_or("");

        self.album_name.set_label(album_title);
    }

    fn broadcast_event(&self, event: &AppEvent) {
        for child in self.children.iter() {
            child.on_event(event);
        }
    }
}

impl Component for Details {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.root
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
