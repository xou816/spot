use gtk::prelude::*;

use crate::app::components::EventListener;
use crate::app::AppEvent;

pub struct MainWindow {
    window: libhandy::ApplicationWindow,
}

impl MainWindow {
    pub fn new(window: libhandy::ApplicationWindow) -> Self {
        window.connect_delete_event(|window, _| {
            window.hide();
            Inhibit(true)
        });
        Self { window }
    }

    fn present(&self) {
        self.window.present();
    }

    fn raise(&self) {
        self.window.show();
    }
}

impl EventListener for MainWindow {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started => self.present(),
            AppEvent::Raised => self.raise(),
            _ => {}
        }
    }
}
