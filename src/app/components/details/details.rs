use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::DetailsModel;

use crate::app::components::{screen_add_css_provider, Component, EventListener, Playlist};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::{AppEvent, BrowserEvent};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/details.ui")]
    pub struct AlbumDetailsWidget {
        #[template_child]
        pub root: TemplateChild<gtk::Widget>,

        #[template_child]
        pub album_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub album_tracks: TemplateChild<gtk::ListView>,

        #[template_child]
        pub album_art: TemplateChild<gtk::Image>,

        #[template_child]
        pub like_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub artist_button: TemplateChild<gtk::LinkButton>,

        #[template_child]
        pub artist_button_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumDetailsWidget {
        const NAME: &'static str = "AlbumDetailsWidget";
        type Type = super::AlbumDetailsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AlbumDetailsWidget {}
    impl WidgetImpl for AlbumDetailsWidget {}
    impl BoxImpl for AlbumDetailsWidget {}
}

glib::wrapper! {
    pub struct AlbumDetailsWidget(ObjectSubclass<imp::AlbumDetailsWidget>) @extends gtk::Widget, gtk::Box;
}

impl AlbumDetailsWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/details.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of AlbumDetailsWidget")
    }

    fn widget(&self) -> &imp::AlbumDetailsWidget {
        imp::AlbumDetailsWidget::from_instance(self)
    }

    fn connect_liked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().like_button.connect_clicked(move |_| f());
    }

    fn connect_artist_clicked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().artist_button.connect_activate_link(move |_| {
            f();
            glib::signal::Inhibit(true)
        });
    }

    fn album_tracks_widget(&self) -> &gtk::ListView {
        self.widget().album_tracks.as_ref()
    }

    fn set_liked(&self, is_liked: bool) {
        self.widget()
            .like_button
            .set_label(if is_liked { "♥" } else { "♡" });
    }

    fn set_loaded(&self) {
        let context = self.style_context();
        context.add_class("details--loaded");
    }

    fn set_album_and_artist(
        &self,
        album: &str,
        artist: &str,
        art_url: Option<&String>,
        worker: &Worker,
    ) {
        let widget = self.widget();

        widget.album_label.set_label(album);
        widget.artist_button_label.set_label(artist);

        let weak_self = self.downgrade();
        if let Some(art_url) = art_url.cloned() {
            worker.send_local_task(async move {
                if let Some(_self) = weak_self.upgrade() {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art_url[..], "jpg", 100, 100)
                        .await;
                    _self.widget().album_art.set_from_pixbuf(pixbuf.as_ref());
                    _self.set_loaded();
                }
            });
        } else {
            self.set_loaded();
        }
    }
}

pub struct Details {
    model: Rc<DetailsModel>,
    worker: Worker,
    widget: AlbumDetailsWidget,
    children: Vec<Box<dyn EventListener>>,
}

impl Details {
    pub fn new(model: Rc<DetailsModel>, worker: Worker) -> Self {
        if model.get_album_info().is_none() {
            model.load_album_info();
        }

        let widget = AlbumDetailsWidget::new();
        let playlist = Box::new(Playlist::new(
            widget.album_tracks_widget().clone(),
            model.clone(),
        ));

        widget.connect_liked(clone!(@weak model => move || {
            model.toggle_save_album();
        }));

        Self {
            model,
            worker,
            widget,
            children: vec![playlist],
        }
    }

    fn update_liked(&self) {
        if let Some(info) = self.model.get_album_info() {
            let is_liked = info.is_liked;
            self.widget.set_liked(is_liked);
        }
    }

    fn update_details(&mut self) {
        if let Some(info) = self.model.get_album_info() {
            let album = &info.title[..];
            let artist = &info.artists_name();

            self.widget.set_liked(info.is_liked);
            self.widget
                .set_album_and_artist(album, artist, info.art.as_ref(), &self.worker);
            self.widget
                .connect_artist_clicked(clone!(@weak self.model as model => move || {
                    model.view_artist();
                }));
        }
    }
}

impl Component for Details {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for Details {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::AlbumDetailsLoaded(id))
                if id == &self.model.id =>
            {
                self.update_details();
            }
            AppEvent::BrowserEvent(BrowserEvent::AlbumSaved(id))
            | AppEvent::BrowserEvent(BrowserEvent::AlbumUnsaved(id))
                if id == &self.model.id =>
            {
                self.update_liked();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
