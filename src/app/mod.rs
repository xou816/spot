use futures::channel::mpsc::Sender;
use std::rc::Rc;
use std::sync::Arc;
use gtk::prelude::*;

pub mod dispatch;
pub use dispatch::{DispatchLoop, ActionDispatcherImpl, ActionDispatcher, Worker};

pub mod components;
use components::*;

pub mod backend;
use backend::Command;
use backend::api::CachedSpotifyClient;

pub mod gtypes;

pub mod models;
use models::*;

mod list_store;
pub use list_store::*;

pub mod state;
pub use state::{AppState, AppModel, AppEvent, AppAction, BrowserEvent, BrowserAction};

pub mod credentials;
pub mod loader;


pub struct App {
    components: Vec<Box<dyn EventListener>>,
    model: Rc<AppModel>
}

impl App {

    fn new(
        model: Rc<AppModel>,
        components: Vec<Box<dyn EventListener>>) -> Self {
        Self { model, components }
    }

    pub fn new_from_builder(
        builder: &gtk::Builder,
        sender: Sender<AppAction>,
        worker: Worker,
        command_sender: Sender<Command>) -> Self {

        let state = AppState::new(Vec::new());
        let dispatcher = Box::new(ActionDispatcherImpl::new(sender, worker.clone()));
        let spotify_client = Arc::new(CachedSpotifyClient::new());
        let model = AppModel::new(state, spotify_client);
        let model = Rc::new(model);

        let components: Vec<Box<dyn EventListener>> = vec![
            App::make_playback(builder, Rc::clone(&model), dispatcher.box_clone(), worker.clone()),
            App::make_playlist(builder, Rc::clone(&model), dispatcher.box_clone()),
            App::make_login(builder, dispatcher.box_clone()),
            App::make_navigation(builder, Rc::clone(&model), dispatcher.box_clone(), worker),
            App::make_search_bar(builder, dispatcher),
            App::make_player_notifier(command_sender)
        ];

        App::new(model, components)
    }

    fn make_player_notifier(sender: Sender<Command>) -> Box<PlayerNotifier> {
        Box::new(PlayerNotifier::new(sender))
    }

    fn make_navigation(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker) -> Box<Navigation> {

        let back_btn: gtk::Button = builder.get_object("nav_back").unwrap();
        let stack: gtk::Stack = builder.get_object("browser_stack").unwrap();

        let model = NavigationModel::new(Rc::clone(&app_model), dispatcher.box_clone());
        let browser_factory = BrowserFactory::new(worker.clone(), Rc::clone(&app_model), dispatcher.box_clone());
        let playlist_factory = PlaylistFactory::new(Rc::clone(&app_model), dispatcher.box_clone());
        let details_factory = DetailsFactory::new(Rc::clone(&app_model), worker.clone(), playlist_factory);
        let search_factory = SearchFactory::new(app_model, dispatcher.box_clone(), worker);
        Box::new(Navigation::new(model, back_btn, stack, browser_factory, details_factory, search_factory))

    }

    fn make_login(builder: &gtk::Builder, dispatcher: Box<dyn ActionDispatcher>) -> Box<Login> {
        let parent: gtk::Window = builder.get_object("window").unwrap();
        let dialog: gtk::Dialog = builder.get_object("login").unwrap();
        let username: gtk::Entry = builder.get_object("username").unwrap();
        let password: gtk::Entry = builder.get_object("password").unwrap();
        let login_btn: gtk::Button = builder.get_object("login_btn").unwrap();

        let model = LoginModel::new(dispatcher);
        Box::new(Login::new(parent, dialog, username, password, login_btn, model))
    }

    fn make_playlist(builder: &gtk::Builder, app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Box<Playlist> {
        let listbox: gtk::ListBox = builder.get_object("listbox").unwrap();
        let playlist = PlaylistFactory::new(app_model, dispatcher).get_current_playlist(listbox);
        Box::new(playlist)
    }

    fn make_playback(builder: &gtk::Builder, app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>, worker: Worker) -> Box<Playback> {
        let play_button: gtk::Button = builder.get_object("play_pause").unwrap();
        let image: gtk::Image = builder.get_object("playing_image").unwrap();
        let current_song_info: gtk::Label = builder.get_object("current_song_info").unwrap();
        let next: gtk::Button = builder.get_object("next").unwrap();
        let prev: gtk::Button = builder.get_object("prev").unwrap();
        let seek_bar: gtk::Scale = builder.get_object("seek_bar").unwrap();

        let model = PlaybackModel::new(app_model, dispatcher);
        Box::new(Playback::new(model, worker, play_button, image, current_song_info, next, prev, seek_bar))
    }

    fn make_search_bar(builder: &gtk::Builder, dispatcher: Box<dyn ActionDispatcher>) -> Box<SearchBar> {
        let search_entry: gtk::SearchEntry = builder.get_object("search_entry").unwrap();
        let model = SearchBarModel(dispatcher);
        Box::new(SearchBar::new(model, search_entry))
    }

    fn handle(&mut self, message: AppAction) {
        let events = self.model.update_state(message);

        for event in events.iter() {
            for component in self.components.iter_mut() {
                component.on_event(event);
            }
        }
    }

    pub async fn start(mut self, dispatch_loop: DispatchLoop) {
        let app = &mut self;
        dispatch_loop.attach(move |action| {
            app.handle(action);
        }).await;
    }
}
