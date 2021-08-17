mod component;
mod playback_controls;
mod playback_info;
mod playback_widget;
pub use component::*;

use glib::prelude::*;

pub fn expose_widgets() {
    playback_widget::PlaybackWidget::static_type();
}
