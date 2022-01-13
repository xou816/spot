use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use crate::app::components::utils::{wrap_flowbox_item, Debouncer};
use crate::app::components::{AlbumWidget, ArtistWidget, Component, EventListener};
use crate::app::dispatch::Worker;
use crate::app::models::{AlbumModel, ArtistModel};
use crate::app::state::{AppEvent, BrowserEvent};

use super::SearchResultsModel;
mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/search.ui")]
    pub struct SearchResultsWidget {
        #[template_child]
        pub go_back: TemplateChild<gtk::Button>,

        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,

        #[template_child]
        pub status_page: TemplateChild<libadwaita::StatusPage>,

        #[template_child]
        pub search_results: TemplateChild<gtk::Widget>,

        #[template_child]
        pub albums_results: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub artist_results: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchResultsWidget {
        const NAME: &'static str = "SearchResultsWidget";
        type Type = super::SearchResultsWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SearchResultsWidget {}
    impl WidgetImpl for SearchResultsWidget {}
    impl BoxImpl for SearchResultsWidget {}
}

glib::wrapper! {
    pub struct SearchResultsWidget(ObjectSubclass<imp::SearchResultsWidget>) @extends gtk::Widget, gtk::Box;
}

impl SearchResultsWidget {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of SearchResultsWidget")
    }

    fn widget(&self) -> &imp::SearchResultsWidget {
        imp::SearchResultsWidget::from_instance(self)
    }

    pub fn connect_go_back<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().go_back.connect_clicked(move |_| f());
    }

    pub fn connect_search_updated<F>(&self, f: F)
    where
        F: Fn(String) + 'static,
    {
        self.widget()
            .search_entry
            .connect_changed(clone!(@weak self as _self => move |s| {
                let query = s.text();
                let query = query.as_str();
                _self.widget().status_page.set_visible(query.is_empty());
                _self.widget().search_results.set_visible(!query.is_empty());
                if !query.is_empty() {
                    f(query.to_string());
                }
            }));
    }

    fn bind_albums_results<F>(&self, worker: Worker, store: &gio::ListStore, on_album_pressed: F)
    where
        F: Fn(&String) + Clone + 'static,
    {
        self.widget()
            .albums_results
            .bind_model(Some(store), move |item| {
                wrap_flowbox_item(item, |album_model| {
                    let f = on_album_pressed.clone();
                    let album = AlbumWidget::for_model(album_model, worker.clone());
                    album.connect_album_pressed(clone!(@weak album_model => move |_| {
                        if let Some(id) = album_model.uri().as_ref() {
                            f(id);
                        }
                    }));
                    album
                })
            });
    }

    fn bind_artists_results<F>(&self, worker: Worker, store: &gio::ListStore, on_artist_pressed: F)
    where
        F: Fn(&String) + Clone + 'static,
    {
        self.widget()
            .artist_results
            .bind_model(Some(store), move |item| {
                wrap_flowbox_item(item, |artist_model| {
                    let f = on_artist_pressed.clone();
                    let artist = ArtistWidget::for_model(artist_model, worker.clone());
                    artist.connect_artist_pressed(clone!(@weak artist_model => move |_| {
                        if let Some(id) = artist_model.id().as_ref() {
                            f(id);
                        }
                    }));
                    artist
                })
            });
    }
}

pub struct SearchResults {
    widget: SearchResultsWidget,
    model: Rc<SearchResultsModel>,
    album_results_model: gio::ListStore,
    artist_results_model: gio::ListStore,
    debouncer: Debouncer,
}

impl SearchResults {
    pub fn new(model: SearchResultsModel, worker: Worker) -> Self {
        let model = Rc::new(model);
        let widget = SearchResultsWidget::new();

        let album_results_model = gio::ListStore::new(AlbumModel::static_type());
        let artist_results_model = gio::ListStore::new(ArtistModel::static_type());

        widget.connect_go_back(clone!(@weak model => move || {
            model.go_back();
        }));

        widget.connect_search_updated(clone!(@weak model => move |q| {
            model.search(q);
        }));

        widget.bind_albums_results(
            worker.clone(),
            &album_results_model,
            clone!(@weak model => move |uri| {
                model.open_album(uri);
            }),
        );

        widget.bind_artists_results(
            worker,
            &artist_results_model,
            clone!(@weak model => move |id| {
                model.open_artist(id);
            }),
        );

        Self {
            widget,
            model,
            album_results_model,
            artist_results_model,
            debouncer: Debouncer::new(),
        }
    }

    fn update_results(&self) {
        if let Some(results) = self.model.get_album_results() {
            self.album_results_model.remove_all();
            for album in results.iter() {
                self.album_results_model.append(&AlbumModel::new(
                    &album.artists_name(),
                    &album.title,
                    album.year(),
                    &album.art,
                    &album.id,
                ));
            }
        }
        if let Some(results) = self.model.get_artist_results() {
            self.artist_results_model.remove_all();
            for artist in results.iter() {
                self.artist_results_model.append(&ArtistModel::new(
                    &artist.name,
                    &artist.photo,
                    &artist.id,
                ));
            }
        }
    }

    fn update_search_query(&self) {
        self.debouncer.debounce(
            600,
            clone!(@weak self.model as model => move || model.fetch_results()),
        );
    }
}

impl Component for SearchResults {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.as_ref()
    }
}

impl EventListener for SearchResults {
    fn on_event(&mut self, app_event: &AppEvent) {
        match app_event {
            AppEvent::BrowserEvent(BrowserEvent::SearchUpdated) => {
                self.update_search_query();
            }
            AppEvent::BrowserEvent(BrowserEvent::SearchResultsUpdated) => {
                self.update_results();
            }
            _ => {}
        }
    }
}
