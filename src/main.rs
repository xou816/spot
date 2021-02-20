#[macro_use(clone)]
extern crate glib;
#[macro_use]
extern crate lazy_static;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::SettingsExt;

mod app;
mod config;
pub use config::VERSION;

use crate::app::backend;
use crate::app::dispatch::{spawn_task_handler, DispatchLoop};
use crate::app::{App, AppAction};

fn main() {
    startup();
    let gtk_app = gtk::Application::new(Some("dev.alextren.Spot"), Default::default()).unwrap();
    let builder = gtk::Builder::from_resource("/dev/alextren/Spot/window.ui");

    let context = glib::MainContext::default();
    context.push_thread_default();

    let dispatch_loop = DispatchLoop::new();
    let sender = dispatch_loop.make_dispatcher();
    let app = App::new(
        builder.clone(),
        sender.clone(),
        spawn_task_handler(&context),
    );
    context.spawn_local(app.attach(dispatch_loop));

    gtk_app.connect_activate(move |gtk_app| {
        let window: libhandy::ApplicationWindow = builder.get_object("window").unwrap();
        window.set_application(Some(gtk_app));
        gtk_app.add_window(&window);
        sender.unbounded_send(AppAction::Start).unwrap();
    });

    context.invoke_local(move || {
        gtk_app.run(&std::env::args().collect::<Vec<_>>());
    });

    std::process::exit(0);
}

fn startup() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK"));
    libhandy::init();

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/spot.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    gtk::Settings::get_default()
        .unwrap()
        .set_property_gtk_application_prefer_dark_theme(true);

    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/dev/alextren/Spot/app.css");

    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
