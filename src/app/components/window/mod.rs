use gio::SettingsExt;
use gtk::prelude::*;
use gtk::DialogExt;

use crate::api::_clear_old_cache;
use crate::app::components::EventListener;
use crate::app::{AppEvent, Worker};

const MESSAGE: &str = "The old application cache must be cleared. 
Please check ~/.cache/img and ~/.cache/net for any files you might own before proceeding. 
Do you wish to clear the cache now?";

// see https://github.com/xou816/spot/issues/107
fn _clear_old_cache_warn(window: &gtk::Window, worker: Worker) {
    let settings = gio::Settings::new("dev.alextren.Spot");
    if settings.get_boolean("old-cache-cleared") {
        return;
    }
    let do_clear = move || {
        let _ = settings.set_boolean("old-cache-cleared", true);
        worker.send_task(Box::pin(async {
            _clear_old_cache().await;
        }));
    };

    if cfg!(feature = "warn-cache") {
        let dialog = gtk::MessageDialog::new(
            Some(window),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Question,
            gtk::ButtonsType::YesNo,
            MESSAGE,
        );
        dialog.show_all();
        dialog.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Yes {
                do_clear();
            }
            dialog.close();
        });
    } else {
        do_clear();
    }
}

pub struct MainWindow {
    window: libhandy::ApplicationWindow,
    worker: Worker,
}

impl MainWindow {
    pub fn new(window: libhandy::ApplicationWindow, worker: Worker) -> Self {
        window.connect_delete_event(|window, _| {
            window.hide();
            Inhibit(true)
        });
        Self { window, worker }
    }

    fn start(&self) {
        self.window.present();
        _clear_old_cache_warn(self.window.upcast_ref(), self.worker.clone());
    }

    fn raise(&self) {
        self.window.show();
    }
}

impl EventListener for MainWindow {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => self.start(),
            AppEvent::Raised => self.raise(),
            _ => {}
        }
    }
}
