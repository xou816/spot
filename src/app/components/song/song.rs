use crate::app::components::{screen_add_css_provider, Component};
use crate::app::models::SongModel;
use gio::MenuModel;
use gladis::Gladis;
use gtk::prelude::*;
use gtk::{MenuButtonExt, WidgetExt};

#[derive(Gladis, Clone)]
struct SongWidget {
    root: gtk::Widget,
    song_index: gtk::Label,
    song_title: gtk::Label,
    song_artist: gtk::Label,
    menu_btn: gtk::MenuButton,
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

    pub fn set_menu(&self, menu: Option<&MenuModel>) {
        if menu.is_some() {
            let menu_btn = &self.widget.menu_btn;
            menu_btn.set_menu_model(menu);
            menu_btn
                .get_style_context()
                .add_class("song__menu--enabled");
        }
    }
}

impl Component for Song {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.root
    }
}
