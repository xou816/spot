use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::components::EventListener;
use crate::app::{AppEvent, AppModel};
use crate::settings::WindowGeometry;

thread_local! {
    static WINDOW_GEOMETRY: RefCell<WindowGeometry> = RefCell::new(WindowGeometry {
        width: 0, height: 0, is_maximized: false
    });
}

pub struct MainWindow {
    initial_window_geometry: WindowGeometry,
    window: libadwaita::ApplicationWindow,
}

impl MainWindow {
    pub fn new(
        initial_window_geometry: WindowGeometry,
        app_model: Rc<AppModel>,
        window: libadwaita::ApplicationWindow,
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

        window.connect_default_height_notify(Self::save_window_geometry);
        window.connect_default_width_notify(Self::save_window_geometry);
        window.connect_maximized_notify(Self::save_window_geometry);

        window.connect_unrealize(|_| {
            debug!("saving geometry");
            WINDOW_GEOMETRY.with(|g| g.borrow().save());
        });

        Self {
            initial_window_geometry,
            window,
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
