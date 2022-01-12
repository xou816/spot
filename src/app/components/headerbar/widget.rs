use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use libadwaita::subclass::prelude::BinImpl;

use crate::app::components::labels;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/headerbar.ui")]
    pub struct HeaderBarWidget {
        #[template_child]
        pub main_header: TemplateChild<libadwaita::HeaderBar>,

        #[template_child]
        pub selection_header: TemplateChild<libadwaita::HeaderBar>,

        #[template_child]
        pub go_back: TemplateChild<gtk::Button>,

        #[template_child]
        pub title: TemplateChild<libadwaita::WindowTitle>,

        #[template_child]
        pub selection_title: TemplateChild<libadwaita::WindowTitle>,

        #[template_child]
        pub start_selection: TemplateChild<gtk::Button>,

        #[template_child]
        pub select_all: TemplateChild<gtk::Button>,

        #[template_child]
        pub cancel: TemplateChild<gtk::Button>,

        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HeaderBarWidget {
        const NAME: &'static str = "HeaderBarWidget";
        type Type = super::HeaderBarWidget;
        type ParentType = libadwaita::Bin;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HeaderBarWidget {}

    impl BuildableImpl for HeaderBarWidget {
        fn add_child(
            &self,
            buildable: &Self::Type,
            builder: &gtk::Builder,
            child: &glib::Object,
            type_: Option<&str>,
        ) {
            if Some("root") == type_ {
                self.parent_add_child(buildable, builder, child, type_);
            } else {
                self.main_header
                    .set_title_widget(child.downcast_ref::<gtk::Widget>());
            }
        }
    }

    impl WidgetImpl for HeaderBarWidget {}
    impl BinImpl for HeaderBarWidget {}
    impl WindowImpl for HeaderBarWidget {}
}

glib::wrapper! {
    pub struct HeaderBarWidget(ObjectSubclass<imp::HeaderBarWidget>) @extends gtk::Widget, libadwaita::Bin;
}

impl HeaderBarWidget {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of HeaderBarWidget")
    }

    fn widget(&self) -> &imp::HeaderBarWidget {
        imp::HeaderBarWidget::from_instance(self)
    }

    pub fn connect_selection_start<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().start_selection.connect_clicked(move |_| f());
    }

    pub fn connect_select_all<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().select_all.connect_clicked(move |_| f());
    }

    pub fn connect_selection_cancel<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().cancel.connect_clicked(move |_| f());
    }

    pub fn connect_go_back<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.widget().go_back.connect_clicked(move |_| f());
    }

    pub fn set_can_go_back(&self, can_go_back: bool) {
        self.widget().go_back.set_visible(can_go_back);
    }

    pub fn set_selection_possible(&self, possible: bool) {
        self.widget().start_selection.set_visible(possible);
    }

    pub fn set_select_all_possible(&self, possible: bool) {
        self.widget().select_all.set_visible(possible);
    }

    pub fn set_selection_active(&self, active: bool) {
        if active {
            self.widget()
                .selection_title
                .set_title(&labels::n_songs_selected_label(0));
            self.widget().selection_title.show();
            self.widget().selection_header.show();
        } else {
            self.widget().selection_title.hide();
            self.widget().selection_header.hide();
        }
    }

    pub fn set_selection_count(&self, count: usize) {
        self.widget()
            .selection_title
            .set_title(&labels::n_songs_selected_label(count));
    }

    pub fn add_classes(&self, classes: &[&str]) {
        let context = self.widget().main_header.style_context();
        for &class in classes {
            context.add_class(class);
        }
    }

    pub fn remove_classes(&self, classes: &[&str]) {
        let context = self.widget().main_header.style_context();
        for &class in classes {
            context.remove_class(class);
        }
    }

    pub fn set_title_visible(&self, visible: bool) {
        self.widget().title.set_visible(visible);
    }

    pub fn set_title_and_subtitle(&self, title: &str, subtitle: &str) {
        self.widget().title.set_title(title);
        self.widget().title.set_subtitle(subtitle);
    }

    pub fn set_title(&self, title: Option<&str>) {
        self.widget().title.set_visible(title.is_some());
        if let Some(title) = title {
            self.widget().title.set_title(title);
        }
    }
}
