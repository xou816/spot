mod sidebar_icon_widget;
mod sidebar_item;
mod sidebar_row;

use crate::app::components::display_add_css_provider;
use gtk::prelude::*;
use gtk::ListBox;
pub use sidebar_item::*;
pub use sidebar_row::*;

pub fn build_sidebar_listbox(builder: &gtk::Builder, list_store: &gio::ListStore) -> ListBox {
    display_add_css_provider(resource!("/sidebar_listbox/sidebar.css"));
    let listbox: gtk::ListBox = builder.object("home_listbox").unwrap();
    listbox.bind_model(Some(list_store), move |obj| {
        let item = obj.downcast_ref::<sidebar_item::SideBarItem>().unwrap();
        let row = SideBarRow::new(item);
        row.upcast::<gtk::Widget>()
    });
    listbox.add_css_class("navigation-sidebar");
    listbox.add_css_class("listbox_sidebar");
    listbox
}
