mod sidebar_item;
mod sidebar_row;

use crate::app::components::sidebar_listbox::sidebar_row::SideBarRow;

use crate::app::components::display_add_css_provider;
use gtk::prelude::*;
use gtk::ListBox;
pub use sidebar_item::*;

pub fn build_sidebar_listbox(builder: &gtk::Builder, list_store: &gio::ListStore) -> ListBox {
    display_add_css_provider(resource!("/sidebar_listbox/sidebar.css"));
    let listbox: gtk::ListBox = builder.object("home_listbox").unwrap();
    listbox.bind_model(Some(list_store), move |obj| {
        let item = obj.clone().downcast::<sidebar_item::SideBarItem>().unwrap();
        let t = item.title();
        let title = Option::from(t.as_str());
        let label = gtk::Label::new(Option::from(title));
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_max_width_chars(20);
        let gtk_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        gtk_box.set_tooltip_text(title);
        let row = SideBarRow::new(item.id().as_str());
        if item.grayedout() {
            gtk_box.append(&label);
            row.set_child(Option::from(&gtk_box));
            row.set_activatable(false);
            row.set_selectable(false);
            row.set_sensitive(false);
            label.add_css_class("caption-heading");
            label.add_css_class("item_sidebar");
        } else {
            let icon = gtk::Image::new();
            icon.add_css_class("item_sidebar");
            icon.set_icon_name(Option::from(item.iconname().as_str()));
            gtk_box.append(&icon);
            gtk_box.append(&label);

            row.set_child(Option::from(&gtk_box));
        }
        row.upcast::<gtk::Widget>()
    });
    listbox.add_css_class("navigation-sidebar");
    listbox.add_css_class("listbox_sidebar");
    listbox
}
