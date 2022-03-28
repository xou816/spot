use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use crate::app::components::{
    display_add_css_provider, AlbumWidget, Component, EventListener, Playlist,
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
        pub top_tracks: TemplateChild<gtk::ListView>,

        #[template_child]
        pub artist_releases: TemplateChild<gtk::GridView>,
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
        display_add_css_provider(resource!("/components/artist_details.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of ArtistDetailsWidget")
    }

    fn widget(&self) -> &imp::ArtistDetailsWidget {
        imp::ArtistDetailsWidget::from_instance(self)
    }

    fn top_tracks_widget(&self) -> &gtk::ListView {
        self.widget().top_tracks.as_ref()
    }

    fn set_loaded(&self) {
        let context = self.style_context();
        context.add_class("artist__loaded");
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

    fn set_model<F>(&self, store: &ListStore<AlbumModel>, worker: Worker, on_clicked: F)
    where
        F: Fn(String) + Clone + 'static
    {
        let gridview = &imp::ArtistDetailsWidget::from_instance(self).artist_releases;

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, list_item| {
            list_item.set_child(Some(&AlbumWidget::new()));
            // Onclick is handled by album and this causes two layers of highlight on hover
            list_item.set_activatable(false);
        });

        factory.connect_bind(move |factory, list_item| {
            AlbumWidget::bind_list_item(factory, list_item, worker.clone(), on_clicked.clone());
        });

        factory.connect_unbind(move |factory, list_item| {
            AlbumWidget::unbind_list_item(factory, list_item);
        });

        gridview.set_factory(Some(&factory));
        gridview.set_model(Some(&gtk::NoSelection::new(Some(store.unsafe_store()))));
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
            widget.set_model(
                &*store,
                worker.clone(),
                clone!(@weak model => move |uri| {
                    model.open_album(uri);
                })
            );
        }

        let playlist = Box::new(Playlist::new(
            widget.top_tracks_widget().clone(),
            Rc::clone(&model),
            worker,
        ));

        Self {
            model,
            widget,
            children: vec![playlist],
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
                self.widget.set_loaded();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
