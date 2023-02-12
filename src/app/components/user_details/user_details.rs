use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use crate::app::components::utils::wrap_flowbox_item;
use crate::app::components::{display_add_css_provider, AlbumWidget, Component, EventListener};
use crate::app::{models::*, ListStore};
use crate::app::{AppEvent, BrowserEvent, Worker};

use super::UserDetailsModel;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/user_details.ui")]
    pub struct UserDetailsWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub user_name: TemplateChild<gtk::Label>,

        #[template_child]
        pub user_playlists: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UserDetailsWidget {
        const NAME: &'static str = "UserDetailsWidget";
        type Type = super::UserDetailsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for UserDetailsWidget {}
    impl WidgetImpl for UserDetailsWidget {}
    impl BoxImpl for UserDetailsWidget {}
}

glib::wrapper! {
    pub struct UserDetailsWidget(ObjectSubclass<imp::UserDetailsWidget>) @extends gtk::Widget, gtk::Box;
}

impl UserDetailsWidget {
    fn new() -> Self {
        display_add_css_provider(resource!("/components/user_details.css"));
        glib::Object::new()
    }

    fn set_user_name(&self, name: &str) {
        let context = self.style_context();
        context.add_class("user__loaded");
        self.imp().user_name.set_text(name);
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

    fn bind_user_playlists<F>(&self, worker: Worker, store: &ListStore<AlbumModel>, on_pressed: F)
    where
        F: Fn(String) + Clone + 'static,
    {
        self.imp()
            .user_playlists
            .bind_model(Some(store.unsafe_store()), move |item| {
                wrap_flowbox_item(item, |item: &AlbumModel| {
                    let f = on_pressed.clone();
                    let album = AlbumWidget::for_model(item, worker.clone());
                    album.connect_album_pressed(clone!(@weak item => move |_| {
                        f(item.uri());
                    }));
                    album
                })
            });
    }
}

pub struct UserDetails {
    model: Rc<UserDetailsModel>,
    widget: UserDetailsWidget,
}

impl UserDetails {
    pub fn new(model: UserDetailsModel, worker: Worker) -> Self {
        model.load_user_details(model.id.clone());

        let widget = UserDetailsWidget::new();
        let model = Rc::new(model);

        widget.connect_bottom_edge(clone!(@weak model => move || {
            model.load_more();
        }));

        if let Some(store) = model.get_list_store() {
            widget.bind_user_playlists(
                worker,
                &store,
                clone!(@weak model => move |uri| {
                    model.open_playlist(uri);
                }),
            );
        }

        Self { model, widget }
    }

    fn update_details(&self) {
        if let Some(name) = self.model.get_user_name() {
            self.widget.set_user_name(&name);
        }
    }
}

impl Component for UserDetails {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.as_ref()
    }
}

impl EventListener for UserDetails {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::BrowserEvent(BrowserEvent::UserDetailsUpdated(id))
                if id == &self.model.id =>
            {
                self.update_details();
            }
            _ => {}
        }
    }
}
