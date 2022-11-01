use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::PlaylistDetailsModel;
use crate::app::components::{AlbumHeaderWidget, PlaylistModel};

use crate::app::components::{Component, EventListener, Playlist};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::state::PlaybackEvent;
use crate::app::{AppEvent, BrowserEvent};
use libadwaita::subclass::prelude::BinImpl;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/playlist_details.ui")]
    pub struct PlaylistDetailsWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub header_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub header_widget: TemplateChild<AlbumHeaderWidget>,

        #[template_child]
        pub header_mobile: TemplateChild<AlbumHeaderWidget>,

        #[template_child]
        pub tracks: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistDetailsWidget {
        const NAME: &'static str = "PlaylistDetailsWidget";
        type Type = super::PlaylistDetailsWidget;
        type ParentType = libadwaita::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistDetailsWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.header_mobile.set_centered();
            self.header_mobile.hide_actions();
            self.header_widget.hide_actions();
        }
    }

    impl WidgetImpl for PlaylistDetailsWidget {}
    impl BinImpl for PlaylistDetailsWidget {}
}

glib::wrapper! {
    pub struct PlaylistDetailsWidget(ObjectSubclass<imp::PlaylistDetailsWidget>) @extends gtk::Widget, libadwaita::Bin;
}

impl PlaylistDetailsWidget {
    fn new() -> Self {
        glib::Object::new()
    }

    fn playlist_tracks_widget(&self) -> &gtk::ListView {
        self.imp().tracks.as_ref()
    }

    fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp()
            .scrolled_window
            .connect_edge_reached(move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    f()
                }
            });
    }

    fn set_header_visible(&self, visible: bool) -> bool {
        let widget = self.imp();
        let is_up_to_date = widget.header_revealer.reveals_child() == visible;
        if !is_up_to_date {
            widget.header_revealer.set_reveal_child(visible);
        }
        is_up_to_date
    }

    fn connect_header(&self) {
        self.set_header_visible(true);

        let scroll_controller =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll_controller.connect_scroll(
            clone!(@weak self as _self => @default-return gtk::Inhibit(false), move |_, _, dy| {
                gtk::Inhibit(!_self.set_header_visible(dy < 0f64))
            }),
        );

        let widget = self.imp();
        widget.scrolled_window.add_controller(scroll_controller);
    }

    fn set_loaded(&self) {
        let context = self.style_context();
        context.add_class("container--loaded");
    }

    fn set_album_and_artist(&self, album: &str, artist: &str) {
        self.imp()
            .header_widget
            .set_album_and_artist_and_year(album, artist, None);
        self.imp()
            .header_mobile
            .set_album_and_artist_and_year(album, artist, None);
    }

    fn set_artwork(&self, art: &gdk_pixbuf::Pixbuf) {
        self.imp().header_widget.set_artwork(art);
        self.imp().header_mobile.set_artwork(art);
    }

    fn connect_artist_clicked<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        self.imp().header_widget.connect_artist_clicked(f.clone());
        self.imp().header_mobile.connect_artist_clicked(f);
    }

    fn connect_play<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        self.widget().header_widget.connect_play(f);
    }

    fn set_playing(&self, is_playing: bool) {
        self.widget().header_widget.set_playing(is_playing);
    }
}

pub struct PlaylistDetails {
    model: Rc<PlaylistDetailsModel>,
    worker: Worker,
    widget: PlaylistDetailsWidget,
    children: Vec<Box<dyn EventListener>>,
}

impl PlaylistDetails {
    pub fn new(model: Rc<PlaylistDetailsModel>, worker: Worker) -> Self {
        if model.get_playlist_info().is_none() {
            model.load_playlist_info();
        }
        let widget = PlaylistDetailsWidget::new();
        let playlist = Box::new(Playlist::new(
            widget.playlist_tracks_widget().clone(),
            model.clone(),
            worker.clone(),
        ));

        widget.connect_header();

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more_tracks();
        }));

        widget.connect_artist_clicked(clone!(@weak model => move || {
            model.view_owner();
        }));

        widget.connect_play(clone!(@weak model => move || model.toggle_play_playlist()));

        Self {
            model,
            worker,
            widget,
            children: vec![playlist],
        }
    }

    fn update_details(&self) {
        if let Some(info) = self.model.get_playlist_info() {
            let title = &info.title[..];
            let owner = &info.owner.display_name[..];
            let art_url = info.art.as_ref();

            self.widget.set_album_and_artist(title, owner);

            if let Some(art_url) = art_url.cloned() {
                let widget = self.widget.downgrade();
                self.worker.send_local_task(async move {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art_url[..], "jpg", 320, 320)
                        .await;
                    if let (Some(widget), Some(ref pixbuf)) = (widget.upgrade(), pixbuf) {
                        widget.set_artwork(pixbuf);
                        widget.set_loaded();
                    }
                });
            } else {
                self.widget.set_loaded();
            }
        }
    }

    fn update_playing(&self, is_playing: bool) {
        if !self.model.playlist_is_playing() {
            self.widget.set_playing(false);
            return;
        }
        self.widget.set_playing(is_playing);
    }
}

impl Component for PlaylistDetails {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for PlaylistDetails {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::PlaylistDetailsLoaded(id))
                if id == &self.model.id =>
            {
                self.update_details();
                self.update_playing(true);
            }
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackPaused) => {
                self.update_playing(false);
            }
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackResumed) => {
                self.update_playing(true);
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
