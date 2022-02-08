use crate::app::components::EventListener;
use crate::app::AppEvent;
use glib::ToVariant;

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

    fn show_playlist_created(&self, message: &str, label: &str, id: &str) {
        let toast = libadwaita::Toast::builder()
            .title(message)
            .timeout(4)
            .action_name("app.open_playlist")
            .button_label(label)
            .action_target(&id.to_variant())
            .build();
        self.toast_overlay.add_toast(&toast);
    }
}

impl EventListener for Notification {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::NotificationShown(content) = event {
            self.show(content)
        } else if let AppEvent::PlaylistCreatedNotificationShown(message, label, id) = event {
            self.show_playlist_created(message, label, id)
        }
    }
}
