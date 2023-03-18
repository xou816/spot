use crate::api::CachedSpotifyClient;
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
            App::make_player_notifier(
                Rc::clone(&model),
                &settings,
                Box::new(ActionDispatcherImpl::new(sender.clone(), worker.clone())),
                sender.clone(),
            ),
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
            App::make_selection_toolbar(builder, Rc::clone(model), dispatcher.box_clone()),
            App::make_playback(
                builder,
                Rc::clone(model),
                dispatcher.box_clone(),
                worker.clone(),
            ),
            App::make_login(builder, dispatcher.box_clone(), worker.clone()),
            App::make_navigation(
                builder,
                Rc::clone(model),
                dispatcher.box_clone(),
                worker.clone(),
            ),
            App::make_search_button(builder, dispatcher.box_clone()),
            App::make_user_menu(builder, Rc::clone(model), dispatcher),
            App::make_notification(builder),
        ];

        self.components.append(&mut components);
    }

    fn make_player_notifier(
        app_model: Rc<AppModel>,
        settings: &SpotSettings,
        dispatcher: Box<dyn ActionDispatcher>,
        sender: UnboundedSender<AppAction>,
    ) -> Box<impl EventListener> {
        let api = app_model.get_spotify();
        Box::new(PlayerNotifier::new(
            app_model,
            dispatcher,
            crate::player::start_player_service(settings.player_settings.clone(), sender.clone()),
            crate::connect::start_connect_server(api, sender),
        ))
    }

    fn make_dbus(
        app_model: Rc<AppModel>,
        sender: UnboundedSender<AppAction>,
    ) -> Box<impl EventListener> {
        Box::new(crate::dbus::start_dbus_server(app_model, sender))
    }

    fn make_window(
        settings: &SpotSettings,
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
    ) -> Box<impl EventListener> {
        let window: libadwaita::ApplicationWindow = builder.object("window").unwrap();
        Box::new(MainWindow::new(settings.window.clone(), app_model, window))
    }

    fn make_navigation(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker,
    ) -> Box<Navigation> {
        let leaflet: libadwaita::Leaflet = builder.object("leaflet").unwrap();
        let navigation_stack: gtk::Stack = builder.object("navigation_stack").unwrap();
        let home_listbox: gtk::ListBox = builder.object("home_listbox").unwrap();
        let model = NavigationModel::new(Rc::clone(&app_model), dispatcher.box_clone());
        let screen_factory = ScreenFactory::new(
            Rc::clone(&app_model),
            dispatcher.box_clone(),
            worker,
            leaflet.clone(),
        );
        Box::new(Navigation::new(
            model,
            leaflet,
            navigation_stack,
            home_listbox,
            screen_factory,
        ))
    }

    fn make_login(
        builder: &gtk::Builder,
        dispatcher: Box<dyn ActionDispatcher>,
        worker: Worker,
    ) -> Box<Login> {
        let parent: gtk::Window = builder.object("window").unwrap();
        let model = LoginModel::new(dispatcher, worker);
        Box::new(Login::new(parent, model))
    }

    fn make_selection_toolbar(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<impl EventListener> {
        Box::new(SelectionToolbar::new(
            SelectionToolbarModel::new(app_model, dispatcher),
            builder.object("selection_toolbar").unwrap(),
        ))
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

    fn make_search_button(
        builder: &gtk::Builder,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<SearchButton> {
        let search_button: gtk::Button = builder.object("search_button").unwrap();
        let model = SearchBarModel(dispatcher);
        Box::new(SearchButton::new(model, search_button))
    }

    fn make_user_menu(
        builder: &gtk::Builder,
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
    ) -> Box<UserMenu> {
        let parent: gtk::Window = builder.object("window").unwrap();
        let settings_model = SettingsModel::new(app_model.clone(), dispatcher.box_clone());
        let settings = Settings::new(parent, settings_model);

        let button: gtk::MenuButton = builder.object("user").unwrap();
        let about: gtk::AboutDialog = builder.object("about").unwrap();
        let model = UserMenuModel::new(app_model, dispatcher);
        let user_menu = UserMenu::new(button, settings, about, model);
        Box::new(user_menu)
    }

    fn make_notification(builder: &gtk::Builder) -> Box<Notification> {
        let toast_overlay: libadwaita::ToastOverlay = builder.object("main").unwrap();
        Box::new(Notification::new(toast_overlay))
    }

    fn handle(&mut self, message: AppAction) {
        let starting = matches!(&message, &AppAction::Start);

        let events = self.model.update_state(message);

        if !events.is_empty() && starting {
            self.add_ui_components();
        }

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
