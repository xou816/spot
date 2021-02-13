use gladis::Gladis;
use gtk::prelude::*;
use gtk::ScrolledWindowExt;
use std::rc::Rc;

use crate::app::components::{screen_add_css_provider, Album, Component, EventListener, Playlist};
use crate::app::models::*;
use crate::app::{AppEvent, BrowserEvent, Worker};

use super::ArtistDetailsModel;

#[derive(Clone, Gladis)]
struct ArtistDetailsWidget {
    pub root: gtk::ScrolledWindow,
    pub artist_name: gtk::Label,
    pub top_tracks: gtk::ListBox,
    pub artist_releases: gtk::FlowBox,
}

impl ArtistDetailsWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/artist_details.css"));
        Self::from_resource(resource!("/components/artist_details.ui")).unwrap()
    }
}

pub struct ArtistDetails {
    model: Rc<ArtistDetailsModel>,
    widget: ArtistDetailsWidget,
    children: Vec<Box<dyn EventListener>>,
}

impl ArtistDetails {
    pub fn new(id: String, model: ArtistDetailsModel, worker: Worker) -> Self {
        let widget = ArtistDetailsWidget::new();
        let model = Rc::new(model);

        let weak_model = Rc::downgrade(&model);
        widget.root.connect_edge_reached(move |_, pos| {
            if let (gtk::PositionType::Bottom, Some(model)) = (pos, weak_model.upgrade()) {
                let _ = model.load_more();
            }
        });

        if let Some(store) = model.get_list_store() {
            let model_clone = Rc::clone(&model);

            widget
                .artist_releases
                .bind_model(Some(store.unsafe_store()), move |item| {
                    let item = item.downcast_ref::<AlbumModel>().unwrap();
                    let child = gtk::FlowBoxChild::new();
                    let album = Album::new(item, worker.clone());
                    let weak = Rc::downgrade(&model_clone);
                    album.connect_album_pressed(move |a| {
                        if let (Some(id), Some(m)) = (a.uri().as_ref(), weak.upgrade()) {
                            m.open_album(id);
                        }
                    });
                    child.add(album.get_root_widget());
                    child.show_all();
                    child.upcast::<gtk::Widget>()
                });
        }

        model.load_artist_details(id);

        let playlist = Box::new(Playlist::new(widget.top_tracks.clone(), Rc::clone(&model)));

        Self {
            widget,
            model,
            children: vec![playlist],
        }
    }

    fn update_details(&self) {
        if let Some(name) = self.model.get_artist_name() {
            self.widget.artist_name.set_text(&name);
        }
    }
}

impl Component for ArtistDetails {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.root.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
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
        self.broadcast_event(event);
    }
}
