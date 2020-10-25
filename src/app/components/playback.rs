use gtk::prelude::*;
use gtk::{ImageExt, RangeExt, ScaleExt};
use std::rc::Rc;
use std::cell::{Cell};

use crate::app::{AppEvent, SongDescription};
use crate::app::components::{Component};


pub trait PlaybackModel {
    fn is_playing(&self) -> bool;
    fn current_song(&self) -> Option<SongDescription>;
    fn play_next_song(&self);
    fn play_prev_song(&self);
    fn toggle_playback(&self);
    fn seek_to(&self, position: u32);
}

pub struct Playback {
    play_button: gtk::Button,
    current_song_info: gtk::Label,
    seek_bar: gtk::Scale,
    seek_source_id: Cell<Option<glib::source::SourceId>>,
    model: Rc<dyn PlaybackModel>
}

impl Playback {

    pub fn new(
        play_button: gtk::Button,
        current_song_info: gtk::Label,
        next: gtk::Button,
        prev: gtk::Button,
        seek_bar: gtk::Scale,
        model: Rc<dyn PlaybackModel>) -> Self {

        let weak_model = Rc::downgrade(&model);
        seek_bar.connect_change_value(move |s, _, _| {
            weak_model.upgrade()
                .map(|model| model.seek_to(s.get_value() as u32));
            glib::signal::Inhibit(false)
        });

        seek_bar.connect_format_value(|_, value| {
            let seconds = (value/1000.0) as i32;
            let minutes = seconds.div_euclid(60);
            let seconds = seconds.rem_euclid(60);
            format!("{}:{:02}", minutes, seconds)
        });

        let weak_model = Rc::downgrade(&model);
        play_button.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.toggle_playback());
        });

        let weak_model = Rc::downgrade(&model);
        next.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.play_next_song());
        });

        let weak_model = Rc::downgrade(&model);
        prev.connect_clicked(move |_| {
            weak_model.upgrade()
                .map(|model| model.play_prev_song());
        });

        Self { play_button, current_song_info, seek_bar, seek_source_id: Cell::new(None), model }
    }

    fn toggle_playing(&self) {

        let is_playing = self.model.is_playing();

        self.play_button.get_children().first()
            .and_then(|child| child.downcast_ref::<gtk::Image>())
            .map(|image| {
                image.set_from_icon_name(
                    Some(playback_image(is_playing)),
                    gtk::IconSize::Button);
            })
            .expect("error updating icon");

        let seek_bar = self.seek_bar.clone();
        let new_source = if is_playing {
            Some(gtk::timeout_add_seconds(1, move || {
                let value = seek_bar.get_value();
                seek_bar.set_value(value + 1000.0);
                glib::Continue(true)
            }))
        } else {
            None
        };

        if let Some(previous) = self.seek_source_id.replace(new_source) {
            glib::source_remove(previous);
        }
    }

    fn update_current_info(&self) {

        if let Some(song) = self.model.current_song() {
            let title = glib::markup_escape_text(&song.title);
            let artist = glib::markup_escape_text(&song.artist);
            let label = format!("<b>{}</b>\n{}", title.as_str(), artist.as_str());
            self.current_song_info.set_label(&label[..]);

            let duration = song.duration as f64;
            self.seek_bar.set_range(0.0, duration);
            self.seek_bar.set_value(0.0);
        }
    }
}

impl Component for Playback {

    fn on_event(&self, event: AppEvent) {
        match event {
            AppEvent::TrackPaused|AppEvent::TrackResumed => {
                self.toggle_playing();
            },
            AppEvent::TrackChanged(_) => {
                self.update_current_info();
                self.toggle_playing();
            }
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


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use gtk_test;
    use gtk::ContainerExt;

    struct MockModel {
        toggle_playback_called: Cell<bool>
    }

    impl MockModel {
        fn new() -> Self {
            Self { toggle_playback_called: Cell::new(false) }
        }
    }

    impl PlaybackModel for MockModel {
        fn is_playing(&self) -> bool { false }
        fn current_song(&self) -> Option<SongDescription> { None }
        fn play_next_song(&self) {}
        fn play_prev_song(&self) {}

        fn toggle_playback(&self) {
            self.toggle_playback_called.replace(true);
        }

        fn seek_to(&self, position: u32) {}
    }

    fn make_host_window<F: Fn(&gtk::Window) -> ()>(builder: F) -> gtk::Window {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        builder(&window);
        window.show_all();
        window.activate_focus();
        window
    }

    #[test]
    fn test_tap_play() {

        gtk::init().unwrap();

        let mock_model = Rc::new(MockModel::new());
        let playback = Playback::new(
            gtk::Button::new(),
            gtk::Label::new(None),
            gtk::Button::new(),
            gtk::Button::new(),
            gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0., 1000., 1.),
            Rc::clone(&mock_model) as Rc<dyn PlaybackModel>);

        let play_button = playback.play_button.clone();
        make_host_window(move |w| {
            w.add(&play_button);
        });


        assert!(!mock_model.toggle_playback_called.get());

        gtk_test::click(&playback.play_button);
        gtk_test::wait(300);

        assert!(mock_model.toggle_playback_called.get());
    }

}
