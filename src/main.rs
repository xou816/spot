#[macro_use(clone)]
extern crate glib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use futures::channel::mpsc::UnboundedSender;
use gettextrs::*;
use gio::ApplicationFlags;
use gio::prelude::*;
use gio::SimpleAction;
use gtk::prelude::*;
use libadwaita::ColorScheme;

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
    env_logger::init();
    textdomain("spot")
        .and_then(|_| bindtextdomain("spot", config::LOCALEDIR))
        .and_then(|_| bind_textdomain_codeset("spot", "UTF-8"))
        .expect("Could not setup localization");

    let settings = settings::SpotSettings::new_from_gsettings().unwrap_or_default();
    startup(&settings);
    let gtk_app = gtk::Application::new(Some(config::APPID), ApplicationFlags::HANDLES_OPEN);
    expose_widgets();
    let builder = gtk::Builder::from_resource("/dev/alextren/Spot/window.ui");
    let window: libadwaita::ApplicationWindow = builder.object("window").unwrap();

    if cfg!(debug_assertions) {
        window.style_context().add_class("devel");
        gtk_app.set_resource_base_path(Some("/dev/alextren/Spot"));
    }

    let context = glib::MainContext::default();

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

    gtk_app.connect_open(move |_, targets, _| {
        // TODO: the activate signal isn't called when open is, but window and sender are already moved to the activate closure at this point
        // There should only be one target because %u is used in desktop file
        let target = &targets[0];
        let uri = target.uri().to_string();
        let mut parts = uri.split(':');
        if parts.next().unwrap_or_default() != "spotify" {
            return
        }
        let action = parts.next().unwrap_or_default();
        // Might start with /// because of https://gitlab.gnome.org/GNOME/glib/-/issues/1886/
        let action = action.strip_prefix("///").unwrap_or(action);
        if action.is_empty() {
            return
        }
        let data = parts.next().unwrap_or_default();
        if data.is_empty() {
            return
        }
        match action {
            "artist" => {
                todo!("Handle artist in URI")
            },
            "album" => {
                todo!("Handle album in URI")
            },
            "track" => {
                todo!("Handle track in URI")
            },
            "search" => {
                todo!("Handle search in URI")
            },
            "user" => {
                let user_action = parts.next().unwrap_or_default();
                if user_action.is_empty() {
                    return
                }
                let user_data = parts.next().unwrap_or_default();        
                if user_data.is_empty() {
                    return
                }
                match user_action {
                    "playlist" => {
                        todo!("Handle playlist in URI")
                    }
                    _ => return,
                }
            },
            _ => return,
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
    let manager = libadwaita::StyleManager::default();

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/spot.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    manager.set_color_scheme(if settings.prefers_dark_theme {
        ColorScheme::PreferDark
    } else {
        ColorScheme::PreferLight
    });

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
