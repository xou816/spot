use gladis::Gladis;
use gtk::prelude::*;
use gtk::ScrolledWindowExt;

use std::rc::{Rc, Weak};

use super::BrowserModel;
use crate::app::components::{Album, Component, EventListener};
use crate::app::dispatch::Worker;
use crate::app::models::AlbumModel;
use crate::app::AppEvent;

#[derive(Clone, Gladis)]
struct BrowserWidget {
    pub scrolled_window: gtk::ScrolledWindow,
    pub flowbox: gtk::FlowBox,
}

impl BrowserWidget {
    fn new() -> Self {
        Self::from_resource(resource!("/components/browser.ui")).unwrap()
    }

    fn root(&self) -> &gtk::Widget {
        self.scrolled_window.upcast_ref()
    }
}

pub struct Browser {
    widget: BrowserWidget,
    worker: Worker,
    model: Rc<BrowserModel>,
}

impl Browser {
    pub fn new(worker: Worker, model: BrowserModel) -> Self {
        let model = Rc::new(model);

        let widget = BrowserWidget::new();

        let weak_model = Rc::downgrade(&model);
        widget.scrolled_window.connect_edge_reached(move |_, pos| {
            if let (gtk::PositionType::Bottom, Some(model)) = (pos, weak_model.upgrade()) {
                model.load_more_albums();
            }
        });

        Self {
            widget,
            worker,
            model,
        }
    }

    fn bind_flowbox(&self, store: &gio::ListStore) {
        let weak_model = Rc::downgrade(&self.model);
        let worker_clone = self.worker.clone();

        self.widget.flowbox.bind_model(Some(store), move |item| {
            let item = item.downcast_ref::<AlbumModel>().unwrap();
            let child = create_album_for(item, worker_clone.clone(), weak_model.clone());
            child.show_all();
            child.upcast::<gtk::Widget>()
        });
    }
}

impl EventListener for Browser {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => {
                self.model.refresh_saved_albums();
                self.bind_flowbox(self.model.get_list_store().unwrap().unsafe_store())
            }
            AppEvent::LoginCompleted => {
                self.model.refresh_saved_albums();
            }
            _ => {}
        }
    }
}

impl Component for Browser {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.root()
    }
}

fn create_album_for(
    album_model: &AlbumModel,
    worker: Worker,
    model: Weak<BrowserModel>,
) -> gtk::FlowBoxChild {
    let child = gtk::FlowBoxChild::new();

    let album = Album::new(album_model, worker);
    child.add(album.get_root_widget());

    album.connect_album_pressed(move |a| {
        if let (Some(model), Some(id)) = (model.upgrade(), a.uri()) {
            model.open_album(&id);
        }
    });

    child
}
