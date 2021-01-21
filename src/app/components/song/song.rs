use crate::app::components::{screen_add_css_provider, Component};
use crate::app::models::SongModel;
use gladis::Gladis;
use gtk::prelude::*;

#[derive(Gladis, Clone)]
struct SongWidget {
    root: gtk::Widget,
    song_index: gtk::Label,
    song_title: gtk::Label,
    song_artist: gtk::Label,
    play_button: gtk::Button,
}

impl SongWidget {
    pub fn new() -> Self {
        screen_add_css_provider(resource!("/components/song.css"));
        Self::from_resource(resource!("/components/song.ui")).unwrap()
    }

    fn set_playing(widget: &gtk::Widget, is_playing: bool) {
        let song_class = "song--playing";
        let context = widget.get_style_context();
        if is_playing {
            context.add_class(song_class);
        } else {
            context.remove_class(song_class);
        }
    }
}

pub struct Song {
    widget: SongWidget,
    model: SongModel,
}

impl Song {
    pub fn new(model: SongModel) -> Self {
        let widget = SongWidget::new();

        model
            .bind_property("index", &widget.song_index, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        model
            .bind_property("title", &widget.song_title, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        model
            .bind_property("artist", &widget.song_artist, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        SongWidget::set_playing(&widget.root, model.get_playing());

        model.connect_playing_local(clone!(@weak widget.root as root => move |song| {
            SongWidget::set_playing(&root, song.get_playing());
        }));

        Self { widget, model }
    }

    pub fn connect_play_pressed<F: Fn(&SongModel) + 'static>(&self, f: F) {
        self.widget
            .play_button
            .connect_clicked(clone!(@weak self.model as model => move |_| f(&model)));
    }
}

impl Component for Song {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }
}
