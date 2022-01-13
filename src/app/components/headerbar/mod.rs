mod widget;
pub use widget::*;

mod component;
pub use component::*;

use glib::prelude::*;

pub fn expose_widgets() {
    widget::HeaderBarWidget::static_type();
}
