use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use crate::app::components::{
    screen_add_css_provider, AlbumWidget, Component, EventListener, Playlist,
};
use crate::app::{models::*, ListStore};
use crate::app::{AppEvent, BrowserEvent, Worker};

use super::ArtistDetailsModel;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/artist_details.ui")]
    pub struct ArtistDetailsWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub artist_name: TemplateChild<gtk::Label>,

        #[template_child]
        pub top_tracks: TemplateChild<gtk::ListBox>,

        #[template_child]
        pub artist_releases: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArtistDetailsWidget {
        const NAME: &'static str = "ArtistDetailsWidget";
        type Type = super::ArtistDetailsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArtistDetailsWidget {}
    impl WidgetImpl for ArtistDetailsWidget {}
    impl BoxImpl for ArtistDetailsWidget {}
}

glib::wrapper! {
    pub struct ArtistDetailsWidget(ObjectSubclass<imp::ArtistDetailsWidget>) @extends gtk::Widget, gtk::Box;
}

impl ArtistDetailsWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/artist_details.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of ArtistDetailsWidget")
    }

    fn widget(&self) -> &imp::ArtistDetailsWidget {
        imp::ArtistDetailsWidget::from_instance(self)
    }

    fn top_tracks_widget(&self) -> &gtk::ListBox {
        self.widget().top_tracks.as_ref()
    }

    fn set_artist_name(&self, name: &str) {
        let context = self.style_context();
        context.add_class("artist__loaded");
        self.widget().artist_name.set_text(name);
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

    fn bind_artist_releases<F>(
        &self,
        worker: Worker,
        store: &ListStore<AlbumModel>,
        on_album_pressed: F,
    ) where
        F: Fn(&String) + Clone + 'static,
    {
        self.widget()
            .artist_releases
            .bind_model(Some(store.unsafe_store()), move |item| {
                let item = item.downcast_ref::<AlbumModel>().unwrap();
                let child = gtk::FlowBoxChild::new();
                let album = AlbumWidget::for_model(item, worker.clone());
                let f = on_album_pressed.clone();
                album.connect_album_pressed(clone!(@weak item => move |_| {
                    if let Some(id) = item.uri().as_ref() {
                        f(id);
                    }
                }));
                child.set_child(Some(&album));
                child.upcast::<gtk::Widget>()
            });
    }
}

pub struct ArtistDetails {
    model: Rc<ArtistDetailsModel>,
    widget: ArtistDetailsWidget,
    children: Vec<Box<dyn EventListener>>,
}

impl ArtistDetails {
    pub fn new(model: Rc<ArtistDetailsModel>, worker: Worker) -> Self {
        model.load_artist_details(model.id.clone());

        let widget = ArtistDetailsWidget::new();

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more();
        }));

        if let Some(store) = model.get_list_store() {
            widget.bind_artist_releases(
                worker,
                &*store,
                clone!(@weak model => move |id| {
                    model.open_album(id);
                }),
            );
        }

        let playlist = Box::new(Playlist::new(
            widget.top_tracks_widget().clone(),
            Rc::clone(&model),
        ));

        Self {
            model,
            widget,
            children: vec![playlist],
        }
    }

    fn update_details(&mut self) {
        if let Some(name) = self.model.get_artist_name() {
            self.widget.set_artist_name(&name);
        }
    }
}

impl Component for ArtistDetails {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for ArtistDetails {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::ArtistDetailsUpdated(id))
                if id == &self.model.id =>
            {
                self.update_details();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
