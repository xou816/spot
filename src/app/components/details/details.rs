use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::release_details::ReleaseDetailsWindow;
use super::DetailsModel;

use crate::app::components::{display_add_css_provider, Component, EventListener, Playlist};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::{AppEvent, BrowserEvent};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/details.ui")]
    pub struct AlbumDetailsWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub header_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub album_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub album_tracks: TemplateChild<gtk::ListView>,

        #[template_child]
        pub album_art: TemplateChild<gtk::Image>,

        #[template_child]
        pub like_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub info_button: TemplateChild<gtk::Button>,

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
        display_add_css_provider(resource!("/components/details.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of AlbumDetailsWidget")
    }

    fn widget(&self) -> &imp::AlbumDetailsWidget {
        imp::AlbumDetailsWidget::from_instance(self)
    }

    fn set_header_visible(&self, visible: bool) -> bool {
        let widget = self.widget();
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

        let widget = self.widget();
        widget.scrolled_window.add_controller(&scroll_controller);
    }

    fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget()
            .scrolled_window
            .connect_edge_reached(move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    f()
                }
            });
    }

    fn connect_liked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().like_button.connect_clicked(move |_| f());
    }

    fn connect_info<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().info_button.connect_clicked(move |_| f());
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
            .set_icon_name(if is_liked { "starred-symbolic" } else { "non-starred-symbolic" });
    }

    fn set_loaded(&self) {
        let context = self.style_context();
        context.add_class("details--loaded");
    }

    pub fn set_artwork(&self, art: &gdk_pixbuf::Pixbuf) {
        self.widget().album_art.set_from_pixbuf(Some(art));
    }

    fn set_album_and_artist(&self, album: &str, artist: &str) {
        let widget = self.widget();
        widget.album_label.set_label(album);
        widget.artist_button_label.set_label(artist);
    }
}

pub struct Details {
    model: Rc<DetailsModel>,
    worker: Worker,
    widget: AlbumDetailsWidget,
    modal: ReleaseDetailsWindow,
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

        let modal = ReleaseDetailsWindow::new();

        widget.connect_liked(clone!(@weak model => move || model.toggle_save_album()));

        widget.connect_header();

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more();
        }));

        widget.connect_info(clone!(@weak modal, @weak widget => move || {
            let modal = modal.upcast_ref::<libadwaita::Window>();
            modal.set_modal(true);
            modal.set_transient_for(
                widget
                    .root()
                    .and_then(|r| r.downcast::<gtk::Window>().ok())
                    .as_ref(),
            );
            modal.show();
        }));

        Self {
            model,
            worker,
            widget,
            modal,
            children: vec![playlist],
        }
    }

    fn update_liked(&self) {
        if let Some(info) = self.model.get_album_info() {
            let is_liked = info.description.is_liked;
            self.widget.set_liked(is_liked);
        }
    }

    fn update_details(&mut self) {
        if let Some(album) = self.model.get_album_info() {
            let details = &album.release_details;
            let album = &album.description;

            self.widget.set_liked(album.is_liked);
            self.widget
                .set_album_and_artist(&album.title[..], &album.artists_name());
            self.widget
                .connect_artist_clicked(clone!(@weak self.model as model => move || {
                    model.view_artist();
                }));

            self.modal.set_details(
                &album.title,
                &album.artists_name(),
                &details.label,
                &details.release_date,
                album.songs.len(),
                &album.formatted_time(),
                &details.copyright_text,
            );

            if let Some(art) = album.art.clone() {
                let widget = self.widget.downgrade();
                let modal = self.modal.downgrade();

                self.worker.send_local_task(async move {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art[..], "jpg", 200, 200)
                        .await;
                    if let (Some(widget), Some(modal), Some(ref pixbuf)) =
                        (widget.upgrade(), modal.upgrade(), pixbuf)
                    {
                        widget.set_artwork(pixbuf);
                        widget.set_loaded();
                        modal.set_artwork(pixbuf);
                    }
                });
            } else {
                self.widget.set_loaded();
            }
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
