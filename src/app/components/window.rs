use gtk::prelude::*;

pub struct Window {
    pub widget: gtk::ApplicationWindow
}

impl Window {
    pub fn new(builder: &gtk::Builder) -> Self {
        let widget: gtk::ApplicationWindow = builder
            .get_object("window")
            .expect("Failed to find the window object");
        Self { widget }
    }
}
