use gio::prelude::SettingsExt;
use gtk::prelude::*;
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
    if settings.boolean("old-cache-cleared") {
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
        dialog.present();
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
    window: libadwaita::ApplicationWindow,
    worker: Worker,
}

impl MainWindow {
    pub fn new(
        initial_window_geometry: WindowGeometry,
        app_model: Rc<AppModel>,
        window: libadwaita::ApplicationWindow,
        search_bar: gtk::SearchBar,
        worker: Worker,
    ) -> Self {
        window.connect_close_request(
            clone!(@weak app_model => @default-return gtk::Inhibit(false), move |window| {
                let state = app_model.get_state();
                if state.playback.is_playing() {
                    window.hide();
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            }),
        );

        let window_controller = gtk::EventControllerKey::new();
        window.add_controller(&window_controller);
        window_controller.set_propagation_phase(gtk::PropagationPhase::Bubble);
        window_controller.connect_key_pressed(clone!(@weak search_bar, @weak window => @default-return gtk::Inhibit(false), move |controller, _, _, _| {
            let search_triggered = controller.forward(&search_bar) || search_bar.is_search_mode();
            if search_triggered {
                search_bar.set_search_mode(true);
                gtk::Inhibit(true)
            } else if let Some(child) = window.first_child().as_ref() {
                gtk::Inhibit(controller.forward(child))
            } else {
                gtk::Inhibit(false)
            }
        }));

        window.connect_default_height_notify(Self::save_window_geometry);
        window.connect_default_width_notify(Self::save_window_geometry);
        window.connect_maximized_notify(Self::save_window_geometry);

        window.connect_unrealize(|_| {
            WINDOW_GEOMETRY.with(|g| g.borrow().save());
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
        self.window.present();
    }

    fn save_window_geometry<W: GtkWindowExt>(window: &W) {
        let (width, height) = window.default_size();
        let is_maximized = window.is_maximized();
        WINDOW_GEOMETRY.with(|g| {
            let mut g = g.borrow_mut();
            g.is_maximized = is_maximized;
            if !is_maximized {
                g.width = width;
                g.height = height;
            }
        });
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
