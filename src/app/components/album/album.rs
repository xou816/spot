use crate::app::components::{screen_add_css_provider, Component};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::models::AlbumModel;
use gladis::Gladis;
use gtk::prelude::*;
use gtk::RevealerExt;

#[derive(Gladis, Clone)]
struct AlbumWidget {
    root: gtk::Widget,
    revealer: gtk::Revealer,
    album_label: gtk::Label,
    artist_label: gtk::Label,
    cover_btn: gtk::Button,
    cover_image: gtk::Image,
}

impl AlbumWidget {
    pub fn new() -> Self {
        screen_add_css_provider(resource!("/components/album.css"));
        Self::from_resource(resource!("/components/album_compact.ui")).unwrap()
    }
}

pub struct Album {
    widget: AlbumWidget,
    model: AlbumModel,
}

impl Album {
    pub fn new(album_model: &AlbumModel, worker: Worker) -> Self {
        let widget = AlbumWidget::new();

        let image = widget.cover_image.downgrade();
        let revealer = widget.revealer.downgrade();
        if let Some(url) = album_model.cover_url() {
            worker.send_local_task(async move {
                if let (Some(image), Some(revealer)) = (image.upgrade(), revealer.upgrade()) {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 75, 75).await;
                    image.set_from_pixbuf(result.as_ref());
                    revealer.set_reveal_child(true);
                }
            });
        } else {
            widget.revealer.set_reveal_child(true);
        }

        album_model
            .bind_property("album", &widget.album_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        album_model
            .bind_property("artist", &widget.artist_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        Self {
            widget,
            model: album_model.clone(),
        }
    }

    pub fn connect_album_pressed<F: Fn(&AlbumModel) + 'static>(&self, f: F) {
        self.widget
            .cover_btn
            .connect_clicked(clone!(@weak self.model as model => move |_| {
                f(&model);
            }));
    }
}

impl Component for Album {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }
}
