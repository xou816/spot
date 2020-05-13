use gio::prelude::*;
use gtk::prelude::*;
use gtk::SettingsExt;

use tokio_core::reactor::Core;
use futures::future::{FutureExt, TryFutureExt};

use std::thread;
use futures::channel::mpsc::{Sender, channel};

mod config;
mod app;

use crate::app::{App, AppAction};
use crate::app::backend::{SpotifyPlayer, PlayerAction};

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/spot.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    gtk::Settings::get_default().unwrap().set_property_gtk_application_prefer_dark_theme(true);

    let app = gtk::Application::new(Some("dev.alextren.Spot"), Default::default()).unwrap();
    let builder = gtk::Builder::new_from_resource("/dev/alextren/Spot/window.ui");

    let sender = setup();
    let _dispatcher = App::start(&builder, sender.clone());

    let window = make_window(&builder);
    app.connect_activate(move |app| {
        window.set_application(Some(app));
        app.add_window(&window);
        window.present();
        _dispatcher.send(AppAction::ShowLogin);
    });


    let ret = app.run(&std::env::args().collect::<Vec<_>>());
    std::process::exit(ret);
}

fn make_window(builder: &gtk::Builder) -> gtk::ApplicationWindow {
    builder.get_object("window").unwrap()
}

fn setup() -> Sender<PlayerAction> {

    let (sender, receiver) = channel::<PlayerAction>(0);

    if true {
        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            if let Err(_) = core.run(SpotifyPlayer::new().start(core.handle(), receiver).boxed_local().compat()) {
                println!("Player thread crashed");
            }
        });
    }

    sender
}
