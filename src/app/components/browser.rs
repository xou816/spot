use gtk::prelude::*;
use gtk::{ButtonExt, ScrolledWindowExt};
use gio::prelude::*;

use std::iter::Iterator;
use std::rc::{Rc, Weak};
use std::cell::Ref;

use crate::app::{AppEvent, AlbumDescription, BrowserEvent};
use crate::app::components::{Component};
use super::gtypes::AlbumModel;
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;

const STYLE: &str = "
button.album {
  padding: 0;
  border-radius: 0;
}
";


pub trait BrowserModel {
    fn get_saved_albums(&self) -> Ref<'_, Vec<AlbumDescription>>;
    fn refresh_saved_albums(&self);
    fn load_more_albums(&self);
    fn play_album(&self, album_uri: &str);
}


pub struct Browser {
    browser_model: gio::ListStore,
    model: Rc<dyn BrowserModel>
}

impl Browser {

    pub fn new(flowbox: gtk::FlowBox, scroll_window: gtk::ScrolledWindow, worker: Worker, model: Rc<dyn BrowserModel>) -> Self {

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

        Self { browser_model, model }
    }

    fn set_saved_albums(&self) {
        self.browser_model.remove_all();
        self.append_albums(self.model.get_saved_albums().iter());
    }

    fn append_next_albums(&self, offset: usize) {
        self.append_albums(self.model.get_saved_albums().iter().skip(offset));
    }

    fn append_albums<'a>(&self, albums: impl Iterator<Item=&'a AlbumDescription>) {

        for album in albums {

            let title = glib::markup_escape_text(&album.title);
            let title = format!("<b>{}</b>", title.as_str());

            let artist = glib::markup_escape_text(&album.artist);
            let artist = format!("<small>{}</small>", artist.as_str());

            self.browser_model.append(&AlbumModel::new(
                &artist,
                &title,
                &album.art,
                &album.id
            ));
        }
    }
}

impl Component for Browser {

    fn on_event(&self, event: AppEvent) {
        match event {
            AppEvent::Started|AppEvent::LoginCompleted => {
                self.model.refresh_saved_albums();
            },
            AppEvent::BrowserEvent(BrowserEvent::ContentSet) => {
                self.set_saved_albums();
            },
            AppEvent::BrowserEvent(BrowserEvent::ContentAppended(offset)) => {
                self.append_next_albums(offset);
            }
            _ => {}
        }
    }
}

fn wrapped_label_style(builder: gtk::LabelBuilder) -> gtk::LabelBuilder {
    builder
        .halign(gtk::Align::Center)
        .wrap(true)
        .max_width_chars(25)
        .use_markup(true)
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
                let loader = ImageLoader::new();
                let result = loader.load_remote(&url, "jpg", 180, 180).await;
                image_clone.set_from_pixbuf(result.as_ref());
            });
        }

        let button = gtk::Button::new();
        button.set_relief(gtk::ReliefStyle::None);
        button.set_margin_top(0);
        button.connect_clicked(move |_| {
            if let (Some(model), Some(uri)) = (model.upgrade(), album.uri()) {
                model.play_album(&uri);
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

    let label = gtk::LabelBuilder::new();
    let label = wrapped_label_style(label).build();

    album.bind_property("album", &label, "label")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

    vbox.pack_start(&label, false, false, 3);

    let label = gtk::LabelBuilder::new();
    let label = wrapped_label_style(label).build();

    album.bind_property("artist", &label, "label")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

    vbox.pack_start(&label, false, false, 3);

    child
}
