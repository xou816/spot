use crate::api::{CachedSpotifyClient, SpotifyApiClient};
use crate::settings::SpotSettings;
use futures::channel::mpsc::UnboundedSender;
use std::rc::Rc;
use std::sync::Arc;

pub mod dispatch;
pub use dispatch::{ActionDispatcher, ActionDispatcherImpl, DispatchLoop, Worker};

pub mod components;
use components::*;

pub mod models;

mod list_store;
pub use list_store::*;

pub mod state;
pub use state::{AppAction, AppEvent, AppModel, AppState, BrowserAction, BrowserEvent};

mod batch_loader;
pub use batch_loader::*;

pub mod credentials;
pub mod loader;

pub mod rng;
pub use rng::LazyRandomIndex;

pub struct App {
    settings: SpotSettings,
    builder: gtk::Builder,
    components: Vec<Box<dyn EventListener>>,
    model: Rc<AppModel>,
    sender: UnboundedSender<AppAction>,
    worker: Worker,
}

impl App {
    pub fn new(
        settings: SpotSettings,
        builder: gtk::Builder,
        sender: UnboundedSender<AppAction>,
        worker: Worker,
    ) -> Self {
        let state = AppState::new();
        let spotify_client = Arc::new(CachedSpotifyClient::new());
        let model = Rc::new(AppModel::new(state, spotify_client));

        let components: Vec<Box<dyn EventListener>> = vec![
            App::make_player_notifier(model.get_spotify(), &settings, sender.clone()),
            App::make_dbus(Rc::clone(&model), sender.clone()),
        ];

        Self {
            settings,
            builder,
            components,
            model,
            sender,
            worker,
        }
    }

    fn add_ui_components(&mut self) {
        let builder = &self.builder;
        let model = &self.model;
        let sender = &self.sender;
        let worker = &self.worker;
        let dispatcher = Box::new(ActionDispatcherImpl::new(sender.clone(), worker.clone()));

        let mut components: Vec<Box<dyn EventListener>> = vec![
            App::make_window(&self.settings, builder, Rc::clone(model)),
            App::make_selection_editor(builder, Rc::clone(model), dispatcher.box_clone()),
            App::make_playback(
                builder,
                Rc::clone(model),
                dispatcher.box_clone(),
                worker.clone(),
            ),
            App::make_login(builder, dispatcher.box_clone()),
            App::make_navigation(
                builder,
                Rc::clone(model),
                dispatcher.box_clone(),
                worker.clone(),
            ),
            App::make_search_bar(builder, dispatcher.box_clone()),
            App::make_user_menu(builder, Rc::clone(model), dispatcher.box_clone()),
            App::make_notification(builder, dispatcher),
        ];

        self.components.append(&mut components);
    }

    fn make_player_notifier(
        api: Arc<dyn SpotifyApiClient + Send + Sync>,
        settings: &SpotSettings,
        sender: UnboundedSender<AppAction>,
    ) -> Box<impl EventListener> {
        Box::new(PlayerNotifier::new(
            sender.clone(),
            crate::player::start_player_service(api, settings.player_settings.clone(), sender),
        ))
    }

    fn make_dbus(
        app_model: Rc<AppModel>,
        sender: UnboundedSender<AppAction>,
    ) -> Box<impl EventListener> {
        Box::new(crate::dbus::start_dbus_server(app_model, sender).expect("could not start server"))
    }

    fn make_window(
        settings: &SpotSettings,
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
    ) -> Box<impl EventListener> {
        let window: libadwaita::ApplicationWindow = builder.object("window").unwrap();
        let search_bar: gtk::SearchBar = builder.object("search_bar").unwrap();
        Box::new(MainWindow::new(
            settings.window.clone(),
            app_model,
            window,
            search_bar,
        ))
    }

    fn make_selection_editor(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<impl EventListener> {
        let headerbar: libadwaita::HeaderBar = builder.object("header_bar").unwrap();
        let selection_toggle: gtk::ToggleButton = builder.object("selection_toggle").unwrap();
        let selection_label: gtk::Label = builder.object("selection_label").unwrap();
        let model = SelectionHeadingModel::new(app_model, dispatcher);
        Box::new(SelectionHeading::new(
            model,
            headerbar,
            selection_toggle,
            selection_label,
        ))
    }

    fn make_navigation(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker,
    ) -> Box<Navigation> {
        let back_btn: gtk::Button = builder.object("nav_back").unwrap();
        let leaflet: libadwaita::Leaflet = builder.object("leaflet").unwrap();
        let navigation_stack: gtk::Stack = builder.object("navigation_stack").unwrap();
        let home_stack_sidebar: gtk::StackSidebar = builder.object("home_stack_sidebar").unwrap();

        let model = NavigationModel::new(Rc::clone(&app_model), dispatcher.box_clone());
        let screen_factory =
            ScreenFactory::new(Rc::clone(&app_model), dispatcher.box_clone(), worker);
        Box::new(Navigation::new(
            model,
            leaflet,
            back_btn,
            navigation_stack,
            home_stack_sidebar,
            screen_factory,
        ))
    }

    fn make_login(builder: &gtk::Builder, dispatcher: Box<dyn ActionDispatcher>) -> Box<Login> {
        let parent: gtk::Window = builder.object("window").unwrap();
        let model = LoginModel::new(dispatcher);
        Box::new(Login::new(parent, model))
    }

    fn make_playback(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker,
    ) -> Box<impl EventListener> {
        let model = PlaybackModel::new(app_model, dispatcher);
        Box::new(PlaybackControl::new(
            model,
            builder.object("playback").unwrap(),
            worker,
        ))
    }

    fn make_search_bar(
        builder: &gtk::Builder,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<SearchBar> {
        let search_button: gtk::ToggleButton = builder.object("search_button").unwrap();
        let search_entry: gtk::SearchEntry = builder.object("search_entry").unwrap();
        let search_bar: gtk::SearchBar = builder.object("search_bar").unwrap();
        let model = SearchBarModel(dispatcher);
        Box::new(SearchBar::new(
            model,
            search_button,
            search_bar,
            search_entry,
        ))
    }

    fn make_user_menu(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<UserMenu> {
        let button: gtk::MenuButton = builder.object("user").unwrap();
        let about: gtk::AboutDialog = builder.object("about").unwrap();
        let model = UserMenuModel::new(app_model, dispatcher);
        let user_menu = UserMenu::new(button, about, model);
        Box::new(user_menu)
    }

    fn make_notification(
        builder: &gtk::Builder,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<Notification> {
        let root: gtk::Box = builder.object("notification").unwrap();
        let content: gtk::Label = builder.object("notification_content").unwrap();
        let close: gtk::Button = builder.object("close_notification").unwrap();
        let model = NotificationModel::new(dispatcher);
        Box::new(Notification::new(model, root, content, close))
    }

    fn handle(&mut self, message: AppAction) {
        if let AppAction::Start = message {
            self.add_ui_components();
        }

        let events = self.model.update_state(message);

        for event in events.iter() {
            for component in self.components.iter_mut() {
                component.on_event(event);
            }
        }
    }

    pub async fn attach(mut self, dispatch_loop: DispatchLoop) {
        let app = &mut self;
        dispatch_loop
            .attach(move |action| {
                app.handle(action);
            })
            .await;
    }
}
