use futures::channel::mpsc::{Sender, Receiver};
use librespot::core::spotify_id::SpotifyId;
use std::rc::Rc;
use std::cell::RefCell;

pub mod dispatch;
pub use dispatch::Dispatcher;

pub mod components;
use components::{Component};
use components::{Playback, Playlist, Login};

pub mod backend;
use backend::Command;

pub mod state;
pub use state::{AppState, SongDescription};


#[derive(Clone, Debug)]
pub enum AppAction {
    Play,
    Pause,
    Load(String),
    ShowLogin,
    TryLogin(String, String),
    LoginSuccess(String)
}

pub struct App {
    components: Vec<Box<dyn Component>>,
    state: Rc<RefCell<AppState>>,
    sender: Sender<Command>
}

impl App {

    fn new(
        sender: Sender<Command>,
        state: Rc<RefCell<AppState>>,
        components: Vec<Box<dyn Component>>) -> Self {
        Self { sender, state, components }
    }

    fn handle(&self, message: AppAction) {
        println!("AppAction={:?}", message);

        if let None = self.try_relay_message(message.clone()) {
            println!("Warning! Could not communicate with backend");
        }

        self.update_state(message.clone());

        for component in self.components.iter() {
            component.handle(message.clone());
        }
    }

    fn update_state(&self, message: AppAction) {
        let mut state = self.state.borrow_mut();
        match message {
            AppAction::Play => {
                state.is_playing = true
            },
            AppAction::Pause => {
                state.is_playing = false
            },
            AppAction::Load(uri) => {
                state.is_playing = true;
                state.current_song_uri = Some(uri)
            },
            AppAction::LoginSuccess(token) => {
                state.token = Some(token)
            }
            _ => {}
        };
    }

    fn try_relay_message(&self, message: AppAction) -> Option<()> {
        let mut sender = self.sender.clone();
        match message.clone() {
            AppAction::Play => sender.try_send(Command::PlayerResume).ok(),
            AppAction::Pause => sender.try_send(Command::PlayerPause).ok(),
            AppAction::Load(track) => {
                if let Some(id) = SpotifyId::from_uri(&track).ok() {
                    sender.try_send(Command::PlayerLoad(id)).ok()
                } else {
                    None
                }
            },
            AppAction::TryLogin(username, password) => {
                sender.try_send(Command::Login(username, password)).ok()
            },
            _ => Some(())
        }
    }

    pub fn start(builder: &gtk::Builder, dispatcher: Dispatcher, receiver: glib::Receiver<AppAction>, command_sender: Sender<Command>) {

        let state = Rc::new(RefCell::new(AppState::new(vec![
            SongDescription::new("Sunday Morning", "The Velvet Underground", "spotify:track:11607FzqoipskTsXrwEHnJ"),
            SongDescription::new("I'm Waiting For The Man", "The Velvet Underground", "spotify:track:3fElupNRLRJ0tbUDahPrAb"),
            SongDescription::new("Femme Fatale", "The Velvet Underground", "spotify:track:3PG7BAJG9WkmNOJOlc4uAo")
        ])));

        let app = App::new(command_sender, Rc::clone(&state), vec![
            Box::new(Playback::new(&builder, Rc::clone(&state), dispatcher.clone())),
            Box::new(Playlist::new(&builder, Rc::clone(&state), dispatcher.clone())),
            Box::new(Login::new(&builder, dispatcher.clone()))
        ]);

        receiver.attach(None, move |msg| {
            app.handle(msg);
            glib::Continue(true)
        });
    }
}
