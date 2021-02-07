use gtk::prelude::*;
use gtk::{ImageExt, LabelExt};
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::models::*;
use crate::app::AppEvent;
use crate::app::AppModel;

pub struct PlaybackInfoModel {
    app_model: Rc<AppModel>,
}

impl PlaybackInfoModel {
    pub fn new(app_model: Rc<AppModel>) -> Self {
        Self { app_model }
    }

    pub fn current_song(&self) -> Option<SongDescription> {
        self.app_model.get_state().current_song()
    }
}

pub struct PlaybackInfo {
    model: Rc<PlaybackInfoModel>,
    worker: Worker,
    current_song_image: gtk::Image,
    current_song_image_small: gtk::Image,
    current_song_info: gtk::Label,
}

impl PlaybackInfo {
    pub fn new(
        model: PlaybackInfoModel,
        worker: Worker,
        current_song_image: gtk::Image,
        current_song_image_small: gtk::Image,
        current_song_info: gtk::Label,
    ) -> Self {
        let model = Rc::new(model);
        Self {
            model,
            worker,
            current_song_image,
            current_song_image_small,
            current_song_info,
        }
    }

    fn update_current_info(&self) {
        if let Some(song) = self.model.current_song() {
            let title = glib::markup_escape_text(&song.title);
            let artist = glib::markup_escape_text(&song.artists_name());
            let label = format!("<b>{}</b>\n{}", title.as_str(), artist.as_str());
            self.current_song_info.set_label(&label[..]);

            let image1 = self.current_song_image.downgrade();
            let image2 = self.current_song_image_small.downgrade();

            if let Some(url) = song.art {
                self.worker.send_local_task(async move {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 48, 48).await;
                    if let (Some(image1), Some(image2)) = (image1.upgrade(), image2.upgrade()) {
                        image1.set_from_pixbuf(result.as_ref());
                        image2.set_from_pixbuf(result.as_ref());
                    }
                });
            }
        }
    }
}

impl EventListener for PlaybackInfo {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::TrackChanged(_) = event {
            self.update_current_info();
        }
    }
}
