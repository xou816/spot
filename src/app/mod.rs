use futures::channel::mpsc::{Sender};
use std::rc::Rc;
use std::cell::RefCell;

pub mod dispatch;
pub use dispatch::{DispatchLoop, Dispatcher, Worker};

pub mod components;
use components::{Component, Playback, Playlist, PlaybackModel, PlaylistModel, Login, LoginModel, Player, Browser};

pub mod backend;
use backend::Command;
use backend::api;

pub mod state;
pub use state::{AppState, AppModel, SongDescription};

pub mod credentials;
pub mod loader;


#[derive(Clone, Debug)]
pub enum AppAction {
    Play,
    Pause,
    Load(String),
    LoadPlaylist(Vec<SongDescription>),
    StartLogin,
    TryLogin(String, String),
    LoginSuccess(String),
    Error
}

pub struct App {
    components: Vec<Box<dyn Component>>,
    model: Rc<RefCell<AppModel>>
}

impl App {

    fn new(
        model: Rc<RefCell<AppModel>>,
        components: Vec<Box<dyn Component>>) -> Self {
        Self { model, components }
    }

    pub fn new_from_builder(
        builder: &gtk::Builder,
        dispatcher: Dispatcher,
        worker: Worker,
        command_sender: Sender<Command>) -> Self {

        let state = AppState::new(Vec::new());
        let model = AppModel::new(state, dispatcher.clone());
        let model = Rc::new(RefCell::new(model));

        let components: Vec<Box<dyn Component>> = vec![
            Box::new(Playback::new(builder, Rc::clone(&model) as Rc<RefCell<dyn PlaybackModel>>, worker.clone())),
            Box::new(Playlist::new(builder, Rc::clone(&model) as Rc<RefCell<dyn PlaylistModel>>)),
            Box::new(Login::new(builder, Rc::clone(&model) as Rc<RefCell<dyn LoginModel>>)),
            Box::new(Player::new(command_sender)),
            Box::new(Browser::new(builder, worker.clone()))
        ];

        App::new(model, components)
    }

    fn handle(&self, message: AppAction) {
        println!("AppAction={:?}", message);

        self.update_state(message.clone());

        for component in self.components.iter() {
            component.handle(&message);
        }
    }

    fn update_state(&self, message: AppAction) {
        let mut model = self.model.borrow_mut();
        match message {
            AppAction::Play => {
                model.state.is_playing = true;
            },
            AppAction::Pause => {
                model.state.is_playing = false;
            },
            AppAction::Load(uri) => {
                model.state.is_playing = true;
                model.state.current_song_uri = Some(uri);
            },
            AppAction::LoadPlaylist(tracks) => {
                model.state.playlist = tracks;
            },
            AppAction::LoginSuccess(token) => {
                model.state.token = Some(token.clone());
                model.dispatcher.dispatch_async(Box::pin(async {
                    if let Some(tracks) = api::get_album(token, "4xwx0x7k6c5VuThz5qVqmV").await {
                        AppAction::LoadPlaylist(tracks)
                    } else {
                        AppAction::Error
                    }
                }));
            }
            _ => {}
        };
    }

    pub async fn start(self, dispatch_loop: DispatchLoop) {
        dispatch_loop.attach(move |action| {
            self.handle(action);
        }).await;
    }
}
