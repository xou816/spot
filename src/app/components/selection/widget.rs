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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SelectionToolbarWidget {
        const NAME: &'static str = "SelectionToolbarWidget";
        type Type = super::SelectionToolbarWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SelectionToolbarWidget {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
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
    fn widget(&self) -> &imp::SelectionToolbarWidget {
        imp::SelectionToolbarWidget::from_instance(self)
    }

    pub fn connect_move_down<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().move_down.connect_clicked(move |_| f());
    }

    pub fn connect_move_up<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().move_up.connect_clicked(move |_| f());
    }

    pub fn connect_queue<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().queue.connect_clicked(move |_| f());
    }

    pub fn connect_remove<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().remove.connect_clicked(move |_| f());
    }

    pub fn set_move(&self, state: SelectionToolState) {
        self.widget().move_up.set_sensitive(state.sensitive());
        self.widget().move_up.set_visible(state.visible());
        self.widget().move_down.set_sensitive(state.sensitive());
        self.widget().move_down.set_visible(state.visible());
    }

    pub fn set_queue(&self, state: SelectionToolState) {
        self.widget().queue.set_sensitive(state.sensitive());
        self.widget().queue.set_visible(state.visible());
    }

    pub fn set_add(&self, state: SelectionToolState) {
        self.widget().add.set_sensitive(state.sensitive());
        self.widget().add.set_visible(state.visible());
    }

    pub fn set_remove(&self, state: SelectionToolState) {
        self.widget().remove.set_sensitive(state.sensitive());
        self.widget().remove.set_visible(state.visible());
    }

    pub fn set_visible(&self, visible: bool) {
        gtk::Widget::set_visible(self.upcast_ref(), visible);
        self.widget().action_bar.set_revealed(visible);
    }

    pub fn connect_playlists<F>(&self, playlists: &[PlaylistSummary], on_playlist_selected: F)
    where
        F: Fn(&str) + Clone + 'static,
    {
        let menu = gio::Menu::new();
        let action_group = SimpleActionGroup::new();

        for PlaylistSummary { title, id } in playlists {
            let action_name = format!("playlist_{}", &id);

            action_group.add_action(&{
                let id = id.clone();
                let action = SimpleAction::new(&action_name, None);
                let f = on_playlist_selected.clone();
                action.connect_activate(move |_, _| f(&id));
                action
            });

            menu.append(
                Some(&labels::add_to_playlist_label(title)),
                Some(&format!("add_to.{}", action_name)),
            );
        }

        let popover = gtk::PopoverMenu::from_model(Some(&menu));
        self.widget().add.set_popover(Some(&popover));
        self.widget()
            .add
            .insert_action_group("add_to", Some(&action_group));
    }
}
