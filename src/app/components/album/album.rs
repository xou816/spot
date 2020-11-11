use gtk::prelude::*;
use gladis::Gladis;
use crate::app::loader::ImageLoader;
use crate::app::dispatch::Worker;
use crate::app::components::{Component, screen_add_css_provider, gtypes::AlbumModel};

#[derive(Gladis, Clone)]
struct AlbumWidget {
    root: gtk::Widget,
    album_label: gtk::Label,
    artist_label: gtk::Label,
    cover_btn: gtk::Button,
    cover_image: gtk::Image
}

impl AlbumWidget {

    pub fn new() -> Self {
        screen_add_css_provider(resource!("/components/album.css"));
        Self::from_resource(resource!("/components/album.ui")).unwrap()
    }
}

pub struct Album {
    widget: AlbumWidget,
    model: AlbumModel
}

impl Album {

    pub fn new(album_model: &AlbumModel, worker: Worker) -> Self {
        let widget = AlbumWidget::new();

        let image = widget.cover_image.clone();
        if let Some(url) = album_model.cover_url() {
            worker.send_task(async move {
                let loader = ImageLoader::new();
                let result = loader.load_remote(&url, "jpg", 180, 180).await;
                image.set_from_pixbuf(result.as_ref());
            });
        }

        album_model.bind_property("album", &widget.album_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        album_model.bind_property("artist", &widget.artist_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        Self { widget, model: album_model.clone() }
    }

    pub fn connect_album_pressed<F: Fn(&AlbumModel) + 'static>(&self, f: F) {
        let model_clone = self.model.clone();
        self.widget.cover_btn.connect_clicked(move |_| f(&model_clone));
    }
}

impl Component for Album {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }
}
