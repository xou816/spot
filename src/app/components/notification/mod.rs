use crate::app::components::EventListener;
use crate::app::AppEvent;
use gettextrs::*;
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

    fn show_playlist_created(&self, id: &str) {
        // translators: This is a notification that pop ups when a new playlist is created. It includes the name of that playlist.
        let message = gettext("New playlist created.");
        // translators: This is a label in the notification shown after creating a new playlist. If it is clicked, the new playlist will be opened.
        let label = gettext("View");
        let toast = libadwaita::Toast::builder()
            .title(&message)
            .timeout(4)
            .action_name("app.open_playlist")
            .button_label(&label)
            .action_target(&id.to_variant())
            .build();
        self.toast_overlay.add_toast(&toast);
    }
}

impl EventListener for Notification {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::NotificationShown(content) = event {
            self.show(content)
        } else if let AppEvent::PlaylistCreatedNotificationShown(id) = event {
            self.show_playlist_created(id)
        }
    }
}
