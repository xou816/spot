use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::album_header::AlbumHeaderWidget;
use super::release_details::ReleaseDetailsWindow;
use super::DetailsModel;

use crate::app::components::{
    Component, EventListener, HeaderBarComponent, HeaderBarWidget, Playlist,
};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::{AppEvent, BrowserEvent};

mod imp {

    use libadwaita::subclass::prelude::BinImpl;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/details.ui")]
    pub struct AlbumDetailsWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub headerbar: TemplateChild<HeaderBarWidget>,

        #[template_child]
        pub header_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub header_widget: TemplateChild<AlbumHeaderWidget>,

        #[template_child]
        pub header_mobile: TemplateChild<AlbumHeaderWidget>,

        #[template_child]
        pub album_tracks: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumDetailsWidget {
        const NAME: &'static str = "AlbumDetailsWidget";
        type Type = super::AlbumDetailsWidget;
        type ParentType = libadwaita::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AlbumDetailsWidget {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.header_mobile.set_centered();
            self.headerbar.add_classes(&["details__headerbar"]);
        }
    }

    impl WidgetImpl for AlbumDetailsWidget {}
    impl BinImpl for AlbumDetailsWidget {}
}

glib::wrapper! {
    pub struct AlbumDetailsWidget(ObjectSubclass<imp::AlbumDetailsWidget>) @extends gtk::Widget, libadwaita::Bin;
}

impl AlbumDetailsWidget {
    fn new() -> Self {
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
            widget.headerbar.set_title_visible(!visible);
            if visible {
                widget.headerbar.add_classes(&["flat"]);
            } else {
                widget.headerbar.remove_classes(&["flat"]);
            }
        }
        is_up_to_date
    }

    fn connect_header_visibility(&self) {
        self.set_header_visible(true);

        let scroll_controller =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll_controller.connect_scroll(
            clone!(@weak self as _self => @default-return gtk::Inhibit(false), move |_, _, dy| {
                let visible = dy < 0f64;
                gtk::Inhibit(!_self.set_header_visible(visible))
            }),
        );

        let swipe_controller = gtk::GestureSwipe::new();
        swipe_controller.set_touch_only(true);
        swipe_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        swipe_controller.connect_swipe(clone!(@weak self as _self => move |_, _, dy| {
            let visible = dy >= 0f64;
            _self.set_header_visible(visible);
        }));

        self.widget()
            .scrolled_window
            .add_controller(&scroll_controller);
        self.add_controller(&swipe_controller);
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

    fn headerbar_widget(&self) -> &HeaderBarWidget {
        self.widget().headerbar.as_ref()
    }

    fn album_tracks_widget(&self) -> &gtk::ListView {
        self.widget().album_tracks.as_ref()
    }

    fn set_loaded(&self) {
        let context = self.style_context();
        context.add_class("container--loaded");
    }

    fn connect_liked<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        self.widget().header_widget.connect_liked(f.clone());
        self.widget().header_mobile.connect_liked(f);
    }

    fn connect_info<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        self.widget().header_widget.connect_info(f.clone());
        self.widget().header_mobile.connect_info(f);
    }

    fn set_liked(&self, is_liked: bool) {
        self.widget().header_widget.set_liked(is_liked);
        self.widget().header_mobile.set_liked(is_liked);
    }

    fn set_album_and_artist_and_year(&self, album: &str, artist: &str, year: Option<u32>) {
        self.widget()
            .header_widget
            .set_album_and_artist_and_year(album, artist, year);
        self.widget()
            .header_mobile
            .set_album_and_artist_and_year(album, artist, year);
        self.widget()
            .headerbar
            .set_title_and_subtitle(album, artist);
    }

    fn set_artwork(&self, art: &gdk_pixbuf::Pixbuf) {
        self.widget().header_widget.set_artwork(art);
        self.widget().header_mobile.set_artwork(art);
    }

    fn connect_artist_clicked<F>(&self, f: F)
    where
        F: Fn() + Clone + 'static,
    {
        self.widget()
            .header_widget
            .connect_artist_clicked(f.clone());
        self.widget().header_mobile.connect_artist_clicked(f);
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
            worker.clone(),
            false,
        ));

        let headerbar_widget = widget.headerbar_widget();
        let headerbar = Box::new(HeaderBarComponent::new(
            headerbar_widget.clone(),
            model.to_headerbar_model(),
        ));

        let modal = ReleaseDetailsWindow::new();

        widget.connect_liked(clone!(@weak model => move || model.toggle_save_album()));

        widget.connect_header_visibility();

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
            children: vec![playlist, headerbar],
        }
    }

    fn update_liked(&self) {
        if let Some(info) = self.model.get_album_info() {
            let is_liked = info.description.is_liked;
            self.widget.set_liked(is_liked);
            self.widget.set_liked(is_liked);
        }
    }

    fn update_details(&mut self) {
        if let Some(album) = self.model.get_album_info() {
            let details = &album.release_details;
            let album = &album.description;

            self.widget.set_liked(album.is_liked);

            self.widget.set_album_and_artist_and_year(
                &album.title[..],
                &album.artists_name(),
                album.year(),
            );

            self.widget.connect_artist_clicked(
                clone!(@weak self.model as model => move || model.view_artist()),
            );

            self.modal.set_details(
                &album.title,
                &album.artists_name(),
                &details.label,
                album.release_date.as_ref().unwrap(),
                album.songs.len(),
                &album.formatted_time(),
                &details.copyright_text,
            );

            if let Some(art) = album.art.clone() {
                let widget = self.widget.downgrade();

                self.worker.send_local_task(async move {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art[..], "jpg", 320, 320)
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
