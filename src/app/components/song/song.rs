use gtk::prelude::*;
use gladis::Gladis;
use crate::app::components::{Component, screen_add_css_provider, gtypes::SongModel};

#[derive(Gladis, Clone)]
struct SongWidget {
    root: gtk::Widget,
    song_index: gtk::Label,
    song_title: gtk::Label,
    song_artist: gtk::Label,
    play_button: gtk::Button
}


impl SongWidget {

    pub fn new() -> Self {
        screen_add_css_provider(resource!("/components/song.css"));
        Self::from_resource(resource!("/components/song.ui")).unwrap()
    }
}

pub struct Song {
    widget: SongWidget,
    model: SongModel
}

impl Song {

    pub fn new(model: SongModel) -> Self {
        let widget = SongWidget::new();

        model.bind_property("title", &widget.song_title, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        model.bind_property("artist", &widget.song_artist, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        let widget_clone = widget.song_title.clone();
        model.connect_local("notify::playing", true, move |values| {
            if let Ok(Some(song)) = values[0].get::<SongModel>() {
                let song_class = "song__title--playing";
                let context = widget_clone.get_style_context();
                if song.get_playing() {
                    context.add_class(song_class);
                } else {
                    context.remove_class(song_class);
                }
            }
            None
        }).expect("connect faild");

        Self { widget, model }
    }

    pub fn connect_play_pressed<F: Fn(&SongModel) + 'static>(&self, f: F) {
        let model_clone = self.model.clone();
        self.widget.play_button.connect_clicked(move |_| f(&model_clone));
    }
}

impl Component for Song {

    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }
}
