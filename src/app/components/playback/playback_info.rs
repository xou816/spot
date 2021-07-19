use gettextrs::*;
use gtk::prelude::*;
use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::models::*;
use crate::app::state::{BrowserAction, PlaybackEvent, ScreenName};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel};

pub struct PlaybackInfoModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl PlaybackInfoModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn current_song(&self) -> Option<impl Deref<Target = SongDescription> + '_> {
        self.app_model.map_state_opt(|s| s.playback.current_song())
    }

    fn go_home(&self) {
        self.dispatcher.dispatch(AppAction::ViewNowPlaying);
        self.dispatcher
            .dispatch(BrowserAction::NavigationPopTo(ScreenName::Home).into());
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
        now_playing: gtk::Button,
        now_playing_small: gtk::Button,
        current_song_image: gtk::Image,
        current_song_image_small: gtk::Image,
        current_song_info: gtk::Label,
    ) -> Self {
        let model = Rc::new(model);
        now_playing.connect_clicked(clone!(@weak model => move |_| model.go_home()));
        now_playing_small.connect_clicked(clone!(@weak model => move |_| model.go_home()));
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

            if let Some(url) = song.art.clone() {
                self.worker.send_local_task(async move {
                    let loader = ImageLoader::new();
                    let result = loader.load_remote(&url, "jpg", 48, 48).await;
                    if let (Some(image1), Some(image2)) = (image1.upgrade(), image2.upgrade()) {
                        image1.set_from_pixbuf(result.as_ref());
                        image2.set_from_pixbuf(result.as_ref());
                    }
                });
            }
        } else {
            self.current_song_info
                // translators: Short text displayed instead of a song title when nothing plays
                .set_label(&gettext("No song playing"));
            self.current_song_image
                .set_from_icon_name(Some("emblem-music-symbolic"));
            self.current_song_image_small
                .set_from_icon_name(Some("emblem-music-symbolic"));
        }
    }
}

impl EventListener for PlaybackInfo {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_))
        | AppEvent::PlaybackEvent(PlaybackEvent::PlaybackStopped) = event
        {
            self.update_current_info();
        }
    }
}
