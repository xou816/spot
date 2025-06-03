use gio::prelude::ActionMapExt;
use gio::{SimpleAction, SimpleActionGroup};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::app::components::{display_add_css_provider, labels};
use crate::app::models::PlaylistSummary;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/selection_toolbar.ui")]
    pub struct SelectionToolbarWidget {
        #[template_child]
        pub action_bar: TemplateChild<gtk::ActionBar>,

        #[template_child]
        pub move_up: TemplateChild<gtk::Button>,

        #[template_child]
        pub move_down: TemplateChild<gtk::Button>,

        #[template_child]
        pub add: TemplateChild<gtk::MenuButton>,

        #[template_child]
        pub remove: TemplateChild<gtk::Button>,

        #[template_child]
        pub queue: TemplateChild<gtk::Button>,

        #[template_child]
        pub save: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SelectionToolbarWidget {
        const NAME: &'static str = "SelectionToolbarWidget";
        type Type = super::SelectionToolbarWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SelectionToolbarWidget {
        fn constructed(&self) {
            self.parent_constructed();
            display_add_css_provider(resource!("/components/selection_toolbar.css"));
        }
    }

    impl WidgetImpl for SelectionToolbarWidget {}
    impl BoxImpl for SelectionToolbarWidget {}
}

glib::wrapper! {
    pub struct SelectionToolbarWidget(ObjectSubclass<imp::SelectionToolbarWidget>) @extends gtk::Widget, gtk::Box;
}

#[derive(Debug, Clone, Copy)]
pub enum SelectionToolState {
    Hidden,
    Visible(bool),
}

impl SelectionToolState {
    fn visible(self) -> bool {
        matches!(self, SelectionToolState::Visible(_))
    }

    fn sensitive(self) -> bool {
        matches!(self, SelectionToolState::Visible(true))
    }
}

impl SelectionToolbarWidget {
    pub fn connect_move_down<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().move_down.connect_clicked(move |_| f());
    }

    pub fn connect_move_up<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().move_up.connect_clicked(move |_| f());
    }

    pub fn connect_queue<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().queue.connect_clicked(move |_| f());
    }

    pub fn connect_save<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().save.connect_clicked(move |_| f());
    }

    pub fn connect_remove<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().remove.connect_clicked(move |_| f());
    }

    pub fn set_move(&self, state: SelectionToolState) {
        self.imp().move_up.set_sensitive(state.sensitive());
        self.imp().move_up.set_visible(state.visible());
        self.imp().move_down.set_sensitive(state.sensitive());
        self.imp().move_down.set_visible(state.visible());
    }

    pub fn set_queue(&self, state: SelectionToolState) {
        self.imp().queue.set_sensitive(state.sensitive());
        self.imp().queue.set_visible(state.visible());
    }

    pub fn set_add(&self, state: SelectionToolState) {
        self.imp().add.set_sensitive(state.sensitive());
        self.imp().add.set_visible(state.visible());
    }

    pub fn set_remove(&self, state: SelectionToolState) {
        self.imp().remove.set_sensitive(state.sensitive());
        self.imp().remove.set_visible(state.visible());
    }

    pub fn set_save(&self, state: SelectionToolState) {
        self.imp().save.set_sensitive(state.sensitive());
        self.imp().save.set_visible(state.visible());
    }

    pub fn set_visible(&self, visible: bool) {
        gtk::Widget::set_visible(self.upcast_ref(), visible);
        self.imp().action_bar.set_revealed(visible);
    }

    pub fn connect_playlists<F>(&self, playlists: &[PlaylistSummary], on_playlist_selected: F)
    where
        F: Fn(&str) + Clone + 'static,
    {
        let menu = gio::Menu::new();
        let action_group = SimpleActionGroup::new();

        for PlaylistSummary { title, id } in playlists {
            let action_name = format!("playlist_{id}");

            action_group.add_action(&{
                let id = id.clone();
                let action = SimpleAction::new(&action_name, None);
                let f = on_playlist_selected.clone();
                action.connect_activate(move |_, _| f(&id));
                action
            });

            menu.append(
                Some(&labels::add_to_playlist_label(title)),
                Some(&format!("add_to.{action_name}")),
            );
        }

        let popover = gtk::PopoverMenu::from_model(Some(&menu));
        self.imp().add.set_popover(Some(&popover));
        self.imp()
            .add
            .insert_action_group("add_to", Some(&action_group));
    }
}
