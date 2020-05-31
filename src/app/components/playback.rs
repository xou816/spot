use gtk::prelude::*;
use gtk::ImageExt;
use std::rc::Rc;
use std::cell::RefCell;

use crate::app::{AppAction, SongDescription};
use crate::app::components::{Component};
use crate::app::dispatch::Worker;
use crate::app::loader::load_remote_image;


pub trait PlaybackModel {
    fn is_playing(&self) -> bool;
    fn current_song(&self) -> Option<&SongDescription>;
    fn play_next_song(&self);
    fn play_prev_song(&self);
    fn toggle_playback(&self);
}

pub struct Playback {
    play_button: gtk::Button,
    current_song_info: gtk::Label,
    model: Rc<RefCell<dyn PlaybackModel>>
}

impl Playback {

    pub fn new(
        builder: &gtk::Builder,
        model: Rc<RefCell<dyn PlaybackModel>>,
        worker: Worker) -> Self {

        let play_button: gtk::Button = builder.get_object("play_pause").unwrap();
        let current_song_info: gtk::Label = builder.get_object("current_song_info").unwrap();
        let next: gtk::Button = builder.get_object("next").unwrap();
        let prev: gtk::Button = builder.get_object("prev").unwrap();

        let weak_model = Rc::downgrade(&model);
        play_button.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.borrow().toggle_playback());
        });

        let weak_model = Rc::downgrade(&model);
        next.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.borrow().play_next_song());
        });

        let weak_model = Rc::downgrade(&model);
        prev.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.borrow().play_prev_song());
        });


        let image: gtk::Image = builder.get_object("playing_image").unwrap();
        worker.send_task(async move {
            let url = "https://images-na.ssl-images-amazon.com/images/I/71YJlc9Wb6L._SL1500_.jpg";
            let result = load_remote_image(url, 60, 60).await;
            image.set_from_pixbuf(result.as_ref());
        });

        Self { play_button, current_song_info, model }
    }

    fn toggle_image(&self) {

        let is_playing = self.model.borrow().is_playing();

        self.play_button.get_children().first()
            .and_then(|child| child.downcast_ref::<gtk::Image>())
            .map(|image| {
                image.set_from_icon_name(
                    Some(playback_image(is_playing)),
                    gtk::IconSize::Button);
            })
            .expect("error updating icon");
    }

    fn update_current_info(&self) {

        let model = self.model.borrow();

        if let Some(song) = model.current_song() {
            let label = format!("<b>{}</b>\n{}", &song.title, &song.artist);
            self.current_song_info.set_label(&label[..]);
        }
    }
}

impl Component for Playback {

    fn handle(&self, action: &AppAction) {
        match action {
            AppAction::Play|AppAction::Pause => {
                self.toggle_image();
            },
            AppAction::Load(_) => {
                self.update_current_info();
                self.toggle_image();
            },
            _ => {}
        }
    }
}

fn playback_image(is_playing: bool) -> &'static str {
    if is_playing {
        "media-playback-pause"
    } else {
        "media-playback-start"
    }
}
