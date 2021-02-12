use futures::executor::block_on;
use gladis::Gladis;
use gtk::prelude::*;
use gtk::ButtonExt;
use libhandy::AvatarExt;

use crate::app::components::Component;
use crate::app::loader::ImageLoader;
use crate::app::models::ArtistModel;

#[derive(Gladis, Clone)]
struct ArtistWidget {
    root: gtk::Widget,
    artist: gtk::Label,
    avatar_btn: gtk::Button,
    avatar: libhandy::Avatar,
}

impl ArtistWidget {
    pub fn new() -> Self {
        Self::from_resource(resource!("/components/artist.ui")).unwrap()
    }
}

pub struct Artist {
    widget: ArtistWidget,
    model: ArtistModel,
}

impl Artist {
    pub fn new(artist_model: &ArtistModel) -> Self {
        let widget = ArtistWidget::new();

        let url = artist_model.image_url();
        widget.avatar.set_image_load_func(Some(Box::new(move |_| {
            let url = url.as_ref()?;
            block_on(async move {
                let loader = ImageLoader::new();
                loader.load_remote(url, "jpg", 200, 200).await
            })
        })));

        artist_model
            .bind_property("artist", &widget.artist, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        Self {
            widget,
            model: artist_model.clone(),
        }
    }

    pub fn connect_artist_pressed<F: Fn(&ArtistModel) + 'static>(&self, f: F) {
        self.widget
            .avatar_btn
            .connect_clicked(clone!(@weak self.model as model => move |_| {
                f(&model);
            }));
    }
}

impl Component for Artist {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }
}
