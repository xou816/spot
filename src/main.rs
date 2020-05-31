use gio::prelude::*;
use gtk::prelude::*;
use gtk::SettingsExt;
use glib;

mod config;
mod app;

use crate::app::{App, AppAction};
use crate::app::backend;
use crate::app::dispatch::{DispatchLoop, LocalTaskLoop};

fn main() {

    setup_gtk();

    let app = gtk::Application::new(Some("dev.alextren.Spot"), Default::default()).unwrap();
    let builder = gtk::Builder::new_from_resource("/dev/alextren/Spot/window.ui");

    let context = glib::MainContext::default();
    context.push_thread_default();

    let dispatch_loop = DispatchLoop::new();
    let dispatcher = dispatch_loop.make_dispatcher();

    let tasks = LocalTaskLoop::new();

    let player_sender = backend::start_player_service(dispatcher.clone());

    context.spawn_local(
        App::new_from_builder(&builder, dispatcher.clone(), tasks.make_worker(), player_sender)
            .start(dispatch_loop));

    context.spawn_local(tasks.attach());

    let window = make_window(&builder);
    app.connect_activate(move |app| {
        window.set_application(Some(app));
        app.add_window(&window);
        window.present();
        // dispatcher.dispatch(AppAction::StartLogin);
    });


    context.invoke_local(move || {
        app.run(&std::env::args().collect::<Vec<_>>());
    });

    std::process::exit(0);
}

fn setup_gtk() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/spot.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    gtk::Settings::get_default().unwrap().set_property_gtk_application_prefer_dark_theme(true);
}

fn make_window(builder: &gtk::Builder) -> gtk::ApplicationWindow {
    builder.get_object("window").unwrap()
}
