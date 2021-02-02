#[macro_use(clone)]
extern crate glib;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::SettingsExt;
use libhandy;

mod app;
mod config;
pub use config::VERSION;

use crate::app::backend;
use crate::app::dispatch::{spawn_task_handler, DispatchLoop};
use crate::app::{App, AppAction};

fn main() {
    setup_gtk();

    let app = gtk::Application::new(Some("dev.alextren.Spot"), Default::default()).unwrap();
    let builder = gtk::Builder::from_resource("/dev/alextren/Spot/window.ui");

    let context = glib::MainContext::default();
    context.push_thread_default();

    let dispatch_loop = DispatchLoop::new();
    let sender = dispatch_loop.make_dispatcher();

    context.spawn_local(
        App::new_from_builder(&builder, sender.clone(), spawn_task_handler(&context))
            .start(dispatch_loop),
    );

    let window = make_window(&builder);
    app.connect_activate(move |app| {
        let mut sender = sender.clone();
        window.set_application(Some(app));
        app.add_window(&window);
        window.present();
        sender.try_send(AppAction::Start).unwrap();
    });

    context.invoke_local(move || {
        app.run(&std::env::args().collect::<Vec<_>>());
    });

    std::process::exit(0);
}

fn setup_gtk() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK"));
    libhandy::init();

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/spot.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    gtk::Settings::get_default()
        .unwrap()
        .set_property_gtk_application_prefer_dark_theme(true);
}

fn make_window(builder: &gtk::Builder) -> libhandy::ApplicationWindow {
    let window: libhandy::ApplicationWindow = builder.get_object("window").unwrap();

    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/dev/alextren/Spot/app.css");

    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window
}
