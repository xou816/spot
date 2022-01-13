mod widget;

mod component;
pub use component::*;

use glib::prelude::*;

pub fn expose_widgets() {
    widget::SelectionToolbarWidget::static_type();
}
