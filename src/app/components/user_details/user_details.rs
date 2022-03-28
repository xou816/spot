use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

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
        pub user_playlists: TemplateChild<gtk::GridView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UserDetailsWidget {
        const NAME: &'static str = "UserDetailsWidget";
        type Type = super::UserDetailsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
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
        glib::Object::new(&[]).expect("Failed to create an instance of UserDetailsWidget")
    }

    fn widget(&self) -> &imp::UserDetailsWidget {
        imp::UserDetailsWidget::from_instance(self)
    }

    fn set_user_name(&self, name: &str) {
        let context = self.style_context();
        context.add_class("user__loaded");
        self.widget().user_name.set_text(name);
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
        let gridview = &imp::UserDetailsWidget::from_instance(self).user_playlists;

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
            widget.set_model(
                &*store,
                worker.clone(),
                clone!(@weak model => move |uri| {
                    model.open_playlist(uri);
                })
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
