use crate::app::components::EventListener;
use crate::app::AppEvent;

pub struct Notification {
    toast_overlay: libadwaita::ToastOverlay,
}

impl Notification {
    pub fn new(toast_overlay: libadwaita::ToastOverlay) -> Self {
        Self { toast_overlay }
    }

    fn show(&self, content: &str) {
        let toast = libadwaita::Toast::builder()
            .title(content)
            .timeout(4)
            .build();
        self.toast_overlay.add_toast(&toast);
    }
}

impl EventListener for Notification {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::NotificationShown(content) = event {
            self.show(content)
        }
    }
}
