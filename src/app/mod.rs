use futures::sync::mpsc::Sender;
use librespot::core::spotify_id::SpotifyId;
use std::rc::Rc;
use std::cell::RefCell;

pub mod components;
use components::{Component, Dispatcher};
use components::{Playback, Playlist, PlaybackState, Login};

pub mod backend;
use backend::PlayerAction;

pub struct AppState {
    is_playing: bool
}

impl PlaybackState for AppState {
    fn is_playing(&self) -> bool {
        self.is_playing
    }
}

#[derive(Clone, Debug)]
pub enum AppAction {
    Play,
    Pause,
    Load(String),
    ShowLogin
}

pub struct App {
    components: Vec<Box<dyn Component>>,
    state: Rc<RefCell<AppState>>,
    sender: Sender<PlayerAction>
}

impl App {

    fn new(
        sender: Sender<PlayerAction>,
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
            AppAction::Load(_) => {
                state.is_playing = true
            },
            _ => {}
        };
    }

    fn try_relay_message(&self, message: AppAction) -> Option<()> {
        let mut sender = self.sender.clone();
        match message.clone() {
            AppAction::Play => sender.try_send(PlayerAction::Play).ok(),
            AppAction::Pause => sender.try_send(PlayerAction::Pause).ok(),
            AppAction::Load(track) => {
                if let Some(id) = SpotifyId::from_uri(&track).ok() {
                    sender.try_send(PlayerAction::Load(id)).ok()
                } else {
                    None
                }
            },
            _ => Some(())
        }
    }

    pub fn start(builder: &gtk::Builder, player_sender: Sender<PlayerAction>) -> Dispatcher {

        let (sender, receiver) = glib::MainContext::channel::<AppAction>(glib::PRIORITY_DEFAULT);
        let state = Rc::new(RefCell::new(AppState { is_playing: false }));


        let dispatcher = Dispatcher::new(sender);

        let app = App::new(player_sender, Rc::clone(&state), vec![
            Box::new(Playback::new(&builder, Rc::clone(&state), dispatcher.clone())),
            Box::new(Playlist::new(&builder, dispatcher.clone())),
            Box::new(Login::new(&builder))
        ]);

        receiver.attach(None, move |msg| {
            app.handle(msg);
            glib::Continue(true)
        });

        dispatcher.clone()

    }
}
