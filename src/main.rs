#[macro_use(clone)]
extern crate glib;
#[macro_use]
extern crate lazy_static;

use futures::channel::mpsc::UnboundedSender;
use gettextrs::*;
use gio::prelude::*;
use gio::SimpleAction;
use gtk::prelude::*;

mod api;
mod app;
mod config;
mod dbus;
mod player;
mod settings;
pub use config::VERSION;

use crate::app::components::expose_widgets;
use crate::app::dispatch::{spawn_task_handler, DispatchLoop};
use crate::app::{state::PlaybackAction, App, AppAction, BrowserAction};

fn main() {
    textdomain("spot")
        .and_then(|_| bindtextdomain("spot", config::LOCALEDIR))
        .and_then(|_| bind_textdomain_codeset("spot", "UTF-8"))
        .expect("Could not setup localization");

    let settings = settings::SpotSettings::new_from_gsettings().unwrap_or_default();
    startup(&settings);
    let gtk_app = gtk::Application::new(Some(config::APPID), Default::default());
    expose_widgets();
    let builder = gtk::Builder::from_resource("/dev/alextren/Spot/window.ui");
    let window: libadwaita::ApplicationWindow = builder.object("window").unwrap();
    if cfg!(debug_assertions) {
        window.style_context().add_class("devel");
    }

    let context = glib::MainContext::default();
    context.push_thread_default();

    let dispatch_loop = DispatchLoop::new();
    let sender = dispatch_loop.make_dispatcher();
    register_actions(&gtk_app, sender.clone());

    let app = App::new(
        settings,
        builder,
        sender.clone(),
        spawn_task_handler(&context),
    );
    context.spawn_local(app.attach(dispatch_loop));

    gtk_app.connect_activate(move |gtk_app| {
        if let Some(existing_window) = gtk_app.active_window() {
            existing_window.present();
        } else {
            window.set_application(Some(gtk_app));
            gtk_app.add_window(&window);
            sender.unbounded_send(AppAction::Start).unwrap();
        }
    });

    context.invoke_local(move || {
        gtk_app.run();
    });

    std::process::exit(0);
}

fn startup(settings: &settings::SpotSettings) {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK"));
    libadwaita::init();
    let manager = libadwaita::StyleManager::default().unwrap();

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/spot.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    if settings.prefers_dark_theme {
        manager.set_color_scheme(libadwaita::ColorScheme::PreferDark);
    } else {
        manager.set_color_scheme(libadwaita::ColorScheme::PreferLight);
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/dev/alextren/Spot/app.css");

    gtk::StyleContext::add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn register_actions(app: &gtk::Application, sender: UnboundedSender<AppAction>) {
    let quit = SimpleAction::new("quit", None);
    quit.connect_activate(clone!(@weak app => move |_, _| {
        app.quit();
    }));
    app.add_action(&quit);

    app.add_action(&make_action(
        "toggle_playback",
        PlaybackAction::TogglePlay.into(),
        sender.clone(),
    ));

    app.add_action(&make_action(
        "player_prev",
        PlaybackAction::Previous.into(),
        sender.clone(),
    ));

    app.add_action(&make_action(
        "player_next",
        PlaybackAction::Next.into(),
        sender.clone(),
    ));

    app.add_action(&make_action(
        "nav_pop",
        AppAction::BrowserAction(BrowserAction::NavigationPop),
        sender,
    ));
}

fn make_action(
    name: &str,
    app_action: AppAction,
    sender: UnboundedSender<AppAction>,
) -> SimpleAction {
    let action = SimpleAction::new(name, None);
    action.connect_activate(move |_, _| {
        sender.unbounded_send(app_action.clone()).unwrap();
    });
    action
}
