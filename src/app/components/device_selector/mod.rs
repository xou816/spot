use glib::StaticType;

mod component;
pub use component::*;

mod widget;
pub use widget::*;

pub fn expose_widgets() {
    widget::DeviceSelectorWidget::static_type();
}
