use gio::SettingsExt;
use gtk::prelude::*;
use gtk::DialogExt;
use libhandy::SearchBarExt;
use std::cell::RefCell;
use std::rc::Rc;

use crate::api::_clear_old_cache;
use crate::app::components::EventListener;
use crate::app::{AppEvent, AppModel, Worker};
use crate::settings::WindowGeometry;

thread_local! {
    static WINDOW_GEOMETRY: RefCell<WindowGeometry> = RefCell::new(WindowGeometry {
        width: 0, height: 0, is_maximized: false
    });
}

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
    initial_window_geometry: WindowGeometry,
    window: libhandy::ApplicationWindow,
    worker: Worker,
}

impl MainWindow {
    pub fn new(
        initial_window_geometry: WindowGeometry,
        app_model: Rc<AppModel>,
        window: libhandy::ApplicationWindow,
        search_bar: libhandy::SearchBar,
        worker: Worker,
    ) -> Self {
        window.connect_delete_event(
            clone!(@weak app_model => @default-return Inhibit(false), move |window, _| {
                let state = app_model.get_state();
                if state.playback.is_playing() {
                    window.hide();
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            }),
        );

        window.connect_key_press_event(move |window, event| {
            let search_triggered = search_bar.handle_event(&mut event.clone());
            if !search_triggered {
                Inhibit(window.propagate_key_event(event))
            } else {
                Inhibit(true)
            }
        });

        window.connect_size_allocate(|window, _| {
            let (width, height) = window.get_size();
            let is_maximized = window.is_maximized();
            WINDOW_GEOMETRY.with(|g| {
                let mut g = g.borrow_mut();
                g.is_maximized = is_maximized;
                if !is_maximized {
                    g.width = width;
                    g.height = height;
                }
            });
        });

        window.connect_destroy(|_| {
            WINDOW_GEOMETRY.with(|g| {
                g.borrow().save();
            });
        });

        Self {
            initial_window_geometry,
            window,
            worker,
        }
    }

    fn start(&self) {
        self.window.set_default_size(
            self.initial_window_geometry.width,
            self.initial_window_geometry.height,
        );
        if self.initial_window_geometry.is_maximized {
            self.window.maximize();
        }
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
