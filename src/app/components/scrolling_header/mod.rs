mod widget;
use glib::StaticType;
pub use widget::*;

pub fn expose_widgets() {
    widget::ScrollingHeaderWidget::static_type();
}
