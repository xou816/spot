use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::PlaylistDetailsModel;

use crate::app::components::{screen_add_css_provider, Component, EventListener, Playlist};
use crate::app::dispatch::Worker;
use crate::app::loader::ImageLoader;
use crate::app::{AppEvent, BrowserEvent};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/playlist_details.ui")]
    pub struct PlaylistDetailsWidget {
        #[template_child]
        pub root: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub owner_button: TemplateChild<gtk::LinkButton>,

        #[template_child]
        pub owner_button_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub tracks: TemplateChild<gtk::ListView>,

        #[template_child]
        pub art: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistDetailsWidget {
        const NAME: &'static str = "PlaylistDetailsWidget";
        type Type = super::PlaylistDetailsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistDetailsWidget {}
    impl WidgetImpl for PlaylistDetailsWidget {}
    impl BoxImpl for PlaylistDetailsWidget {}
}

glib::wrapper! {
    pub struct PlaylistDetailsWidget(ObjectSubclass<imp::PlaylistDetailsWidget>) @extends gtk::Widget, gtk::Box;
}

impl PlaylistDetailsWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/playlist_details.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of PlaylistDetailsWidget")
    }

    fn widget(&self) -> &imp::PlaylistDetailsWidget {
        imp::PlaylistDetailsWidget::from_instance(self)
    }

    fn playlist_tracks_widget(&self) -> &gtk::ListView {
        self.widget().tracks.as_ref()
    }

    fn connect_owner_clicked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().owner_button.connect_activate_link(move |_| {
            f();
            glib::signal::Inhibit(true)
        });
    }

    fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::PlaylistDetailsWidget::from_instance(self)
            .root
            .connect_edge_reached(move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    f()
                }
            });
    }

    fn set_loaded(&self) {
        let context = self.style_context();
        context.add_class("playlist_details--loaded");
    }

    fn set_name_and_owner(
        &self,
        name: &str,
        owner: &str,
        art_url: Option<&String>,
        worker: &Worker,
    ) {
        let widget = self.widget();

        widget.name_label.set_label(name);
        widget.owner_button_label.set_label(owner);

        let weak_self = self.downgrade();
        if let Some(art_url) = art_url.cloned() {
            worker.send_local_task(async move {
                if let Some(_self) = weak_self.upgrade() {
                    let pixbuf = ImageLoader::new()
                        .load_remote(&art_url[..], "jpg", 100, 100)
                        .await;
                    _self.widget().art.set_from_pixbuf(pixbuf.as_ref());
                    _self.set_loaded();
                }
            });
        } else {
            self.set_loaded();
        }
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
        ));

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more_tracks();
        }));

        widget.connect_owner_clicked(clone!(@weak model => move || {
            model.view_owner();
        }));

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

            self.widget
                .set_name_and_owner(title, owner, art_url, &self.worker);
        }
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
                self.update_details()
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
