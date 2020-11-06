use futures::channel::mpsc::{Sender};
use std::rc::Rc;
use std::cell::RefCell;
use gtk::prelude::*;

pub mod dispatch;
pub use dispatch::{DispatchLoop, ActionDispatcherImpl, ActionDispatcher, Worker};

pub mod components;
use components::{EventListener, Playback, Playlist, Login, PlayerNotifier, Navigation};

pub mod connect;
use connect::{PlaylistFactory, PlaybackModelImpl, LoginModelImpl, BrowserFactory, DetailsFactory, NavigationModelImpl};

pub mod backend;
use backend::Command;
use backend::api::CachedSpotifyClient;

pub mod models;
pub use models::{SongDescription, AlbumDescription};

pub mod state;
pub use state::{AppState, AppModel};

pub mod browser_state;
pub use browser_state::{BrowserEvent, BrowserAction};

pub mod credentials;
pub mod loader;


#[derive(Clone, Debug)]
pub enum AppAction {
    Play,
    Pause,
    Seek(u32),
    Load(String),
    LoadPlaylist(Vec<SongDescription>),
    Start,
    TryLogin(String, String),
    LoginSuccess(credentials::Credentials),
    Next,
    Previous,
    BrowserAction(BrowserAction)
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Started,
    TrackPaused,
    TrackResumed,
    TrackSeeked(u32),
    LoginStarted(String, String),
    LoginCompleted,
    TrackChanged(String),
    PlaylistChanged,
    BrowserEvent(BrowserEvent)
}

trait UpdatableState {
    type Action;
    type Event;

    fn update_with(&mut self, action: Self::Action) -> Vec<Self::Event>;
}

pub struct App {
    components: Vec<Box<dyn EventListener>>,
    model: Rc<RefCell<AppModel>>
}

impl App {

    fn new(
        model: Rc<RefCell<AppModel>>,
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
        let spotify_client = Rc::new(CachedSpotifyClient::new());
        let model = AppModel::new(state, spotify_client);
        let model = Rc::new(RefCell::new(model));

        let components: Vec<Box<dyn EventListener>> = vec![
            App::make_playback(builder, Rc::clone(&model), dispatcher.box_clone(), worker.clone()),
            App::make_playlist(builder, Rc::clone(&model), dispatcher.box_clone()),
            App::make_login(builder, dispatcher.box_clone()),
            App::make_navigation(builder, Rc::clone(&model), dispatcher, worker),
            App::make_player_notifier(command_sender)
        ];

        App::new(model, components)
    }

    fn make_player_notifier(sender: Sender<Command>) -> Box<PlayerNotifier> {
        Box::new(PlayerNotifier::new(sender))
    }

    fn make_navigation(
        builder: &gtk::Builder,
        app_model: Rc<RefCell<AppModel>>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker) -> Box<Navigation> {

        let back_btn: gtk::Button = builder.get_object("nav_back").unwrap();
        let stack: gtk::Stack = builder.get_object("browser_stack").unwrap();

        let model = NavigationModelImpl::new(dispatcher.box_clone());
        let browser_factory = BrowserFactory::new(worker, Rc::clone(&app_model), dispatcher.box_clone());
        let playlist_factory = PlaylistFactory::new(Rc::clone(&app_model), dispatcher.box_clone());
        let details_factory = DetailsFactory::new(app_model, playlist_factory);
        Box::new(Navigation::new(Rc::new(model), back_btn, stack, browser_factory, details_factory))

    }

    fn make_login(builder: &gtk::Builder, dispatcher: Box<dyn ActionDispatcher>) -> Box<Login> {
        let parent: gtk::Window = builder.get_object("window").unwrap();
        let dialog: gtk::Dialog = builder.get_object("login").unwrap();
        let username: gtk::Entry = builder.get_object("username").unwrap();
        let password: gtk::Entry = builder.get_object("password").unwrap();
        let login_btn: gtk::Button = builder.get_object("login_btn").unwrap();

        let model = Rc::new(LoginModelImpl::new(dispatcher));
        Box::new(Login::new(parent, dialog, username, password, login_btn, model))
    }

    fn make_playlist(builder: &gtk::Builder, app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Box<Playlist> {
        let listbox: gtk::ListBox = builder.get_object("listbox").unwrap();
        let playlist = PlaylistFactory::new(app_model, dispatcher).get_current_playlist(listbox);
        Box::new(playlist)
    }

    fn make_playback(builder: &gtk::Builder, app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>, worker: Worker) -> Box<Playback> {
        let play_button: gtk::Button = builder.get_object("play_pause").unwrap();
        let image: gtk::Image = builder.get_object("playing_image").unwrap();
        let current_song_info: gtk::Label = builder.get_object("current_song_info").unwrap();
        let next: gtk::Button = builder.get_object("next").unwrap();
        let prev: gtk::Button = builder.get_object("prev").unwrap();
        let seek_bar: gtk::Scale = builder.get_object("seek_bar").unwrap();

        let model = Rc::new(PlaybackModelImpl::new(app_model, dispatcher));
        Box::new(Playback::new(model, worker, play_button, image, current_song_info, next, prev, seek_bar))
    }

    fn handle(&self, message: AppAction) {
        let events = {
            let mut model = self.model.borrow_mut();
            model.update_state(message)
        };

        for event in events.iter() {
            for component in self.components.iter() {
                component.on_event(event);
            }
        }
    }

    pub async fn start(self, dispatch_loop: DispatchLoop) {
        dispatch_loop.attach(move |action| {
            self.handle(action);
        }).await;
    }
}
