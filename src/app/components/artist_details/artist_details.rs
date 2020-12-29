use gladis::Gladis;
use gtk::prelude::*;
use std::rc::Rc;

use crate::app::components::{Album, Component, EventListener};
use crate::app::models::*;
use crate::app::{AppEvent, BrowserEvent, Worker};

use super::ArtistDetailsModel;

#[derive(Clone, Gladis)]
struct ArtistDetailsWidget {
    pub root: gtk::Widget,
    pub artist_name: gtk::Label,
    pub artist_albums: gtk::FlowBox,
}

impl ArtistDetailsWidget {
    fn new() -> Self {
        Self::from_resource(resource!("/components/artist_details.ui")).unwrap()
    }
}

pub struct ArtistDetails {
    model: Rc<ArtistDetailsModel>,
    widget: ArtistDetailsWidget,
}

impl ArtistDetails {
    pub fn new(id: String, model: ArtistDetailsModel, worker: Worker) -> Self {
        let widget = ArtistDetailsWidget::new();
        let model = Rc::new(model);

        if let Some(store) = model.get_list_store() {
            let model_clone = Rc::clone(&model);

            widget
                .artist_albums
                .bind_model(Some(store.unsafe_store()), move |item| {
                    let item = item.downcast_ref::<AlbumModel>().unwrap();
                    let child = gtk::FlowBoxChild::new();
                    let album = Album::new(item, worker.clone());
                    let weak = Rc::downgrade(&model_clone);
                    album.connect_album_pressed(move |a| {
                        if let (Some(uri), Some(m)) = (a.uri().as_ref(), weak.upgrade()) {
                            m.open_album(uri);
                        }
                    });
                    child.add(album.get_root_widget());
                    child.show_all();
                    child.upcast::<gtk::Widget>()
                });
        }

        model.load_artist_details(id);

        Self { widget, model }
    }

    fn update_details(&self) {
        if let Some(name) = self.model.get_artist_name() {
            self.widget.artist_name.set_text(&name);
        }
    }
}

impl Component for ArtistDetails {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }
}

impl EventListener for ArtistDetails {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::ArtistDetailsUpdated) => {
                self.update_details();
            }
            _ => {}
        }
    }
}
