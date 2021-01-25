use futures::channel::mpsc::Sender;
use gtk::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

pub mod dispatch;
pub use dispatch::{ActionDispatcher, ActionDispatcherImpl, DispatchLoop, Worker};

pub mod components;
use components::*;

pub mod backend;
use backend::api::CachedSpotifyClient;
use backend::Command;

pub mod dbus;

pub mod gtypes;

pub mod models;
use models::*;

mod list_store;
pub use list_store::*;

pub mod state;
pub use state::{AppAction, AppEvent, AppModel, AppState, BrowserAction, BrowserEvent};

pub mod credentials;
pub mod loader;

pub struct App {
    components: Vec<Box<dyn EventListener>>,
    model: Rc<AppModel>,
}

impl App {
    fn new(model: Rc<AppModel>, components: Vec<Box<dyn EventListener>>) -> Self {
        Self { model, components }
    }

    pub fn new_from_builder(
        builder: &gtk::Builder,
        sender: Sender<AppAction>,
        worker: Worker,
        command_sender: Sender<Command>,
    ) -> Self {
        let state = AppState::new();
        let dispatcher = Box::new(ActionDispatcherImpl::new(sender, worker.clone()));
        let spotify_client = Arc::new(CachedSpotifyClient::new());
        let model = AppModel::new(state, spotify_client);
        let model = Rc::new(model);

        let components: Vec<Box<dyn EventListener>> = vec![
            App::make_playback(
                builder,
                Rc::clone(&model),
                dispatcher.box_clone(),
                worker.clone(),
            ),
            App::make_login(builder, dispatcher.box_clone()),
            App::make_navigation(builder, Rc::clone(&model), dispatcher.box_clone(), worker),
            App::make_search_bar(builder, dispatcher.box_clone()),
            App::make_player_notifier(command_sender),
            App::make_user_menu(builder, Rc::clone(&model), dispatcher.box_clone()),
            App::make_notification(builder, dispatcher),
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
        worker: Worker,
    ) -> Box<Navigation> {
        let back_btn: gtk::Button = builder.get_object("nav_back").unwrap();
        let navigation_stack: gtk::Stack = builder.get_object("navigation_stack").unwrap();
        let home_stack_sidebar: gtk::StackSidebar =
            builder.get_object("home_stack_sidebar").unwrap();

        let model = NavigationModel::new(Rc::clone(&app_model), dispatcher.box_clone());
        let browser_factory = BrowserFactory::new(
            worker.clone(),
            Rc::clone(&app_model),
            dispatcher.box_clone(),
        );
        let playlist_factory = PlaylistFactory::new(Rc::clone(&app_model), dispatcher.box_clone());
        let details_factory = DetailsFactory::new(
            Rc::clone(&app_model),
            dispatcher.box_clone(),
            worker.clone(),
            playlist_factory,
        );
        let search_factory = SearchFactory::new(
            Rc::clone(&app_model),
            dispatcher.box_clone(),
            worker.clone(),
        );
        let artist_details_factory =
            ArtistDetailsFactory::new(Rc::clone(&app_model), dispatcher.box_clone(), worker);
        let now_playing_factory =
            NowPlayingFactory::new(Rc::clone(&app_model), dispatcher.box_clone());
        Box::new(Navigation::new(
            model,
            back_btn,
            navigation_stack,
            home_stack_sidebar,
            browser_factory,
            details_factory,
            search_factory,
            artist_details_factory,
            now_playing_factory,
        ))
    }

    fn make_login(builder: &gtk::Builder, dispatcher: Box<dyn ActionDispatcher>) -> Box<Login> {
        let parent: gtk::Window = builder.get_object("window").unwrap();
        let dialog: gtk::Dialog = builder.get_object("login").unwrap();
        let username: gtk::Entry = builder.get_object("username").unwrap();
        let password: gtk::Entry = builder.get_object("password").unwrap();
        let login_btn: gtk::Button = builder.get_object("login_btn").unwrap();

        let model = LoginModel::new(dispatcher);
        Box::new(Login::new(
            parent, dialog, username, password, login_btn, model,
        ))
    }

    fn make_playback(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker,
    ) -> Box<Playback> {
        let play_button: gtk::Button = builder.get_object("play_pause").unwrap();
        let shuffle_button: gtk::ToggleButton = builder.get_object("shuffle").unwrap();
        let image: gtk::Image = builder.get_object("playing_image").unwrap();
        let current_song_info: gtk::Label = builder.get_object("current_song_info").unwrap();
        let next: gtk::Button = builder.get_object("next").unwrap();
        let prev: gtk::Button = builder.get_object("prev").unwrap();
        let seek_bar: gtk::Scale = builder.get_object("seek_bar").unwrap();
        let track_duration: gtk::Label = builder.get_object("track_duration").unwrap();

        let widget = PlaybackWidget::new(
            play_button,
            shuffle_button,
            image,
            current_song_info,
            seek_bar,
            track_duration,
            next,
            prev,
        );

        let model = PlaybackModel::new(app_model, dispatcher);
        Box::new(Playback::new(model, worker, widget))
    }

    fn make_search_bar(
        builder: &gtk::Builder,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<SearchBar> {
        let search_entry: gtk::SearchEntry = builder.get_object("search_entry").unwrap();
        let model = SearchBarModel(dispatcher);
        Box::new(SearchBar::new(model, search_entry))
    }

    fn make_user_menu(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<UserMenu> {
        let button: gtk::MenuButton = builder.get_object("user").unwrap();
        let model = UserMenuModel::new(app_model, dispatcher);
        let user_menu = UserMenu::new(button, model);
        Box::new(user_menu)
    }

    fn make_notification(
        builder: &gtk::Builder,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<Notification> {
        let root: gtk::Box = builder.get_object("notification").unwrap();
        let content: gtk::Label = builder.get_object("notification_content").unwrap();
        let close: gtk::Button = builder.get_object("close_notification").unwrap();
        let model = NotificationModel::new(dispatcher);
        Box::new(Notification::new(model, root, content, close))
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
        dispatch_loop
            .attach(move |action| {
                app.handle(action);
            })
            .await;
    }
}
