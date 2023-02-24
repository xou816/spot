use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/scrolling_header.ui")]
    pub struct ScrollingHeaderWidget {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScrollingHeaderWidget {
        const NAME: &'static str = "ScrollingHeaderWidget";
        type Type = super::ScrollingHeaderWidget;
        type ParentType = gtk::Box;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScrollingHeaderWidget {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl BuildableImpl for ScrollingHeaderWidget {
        fn add_child(&self, builder: &gtk::Builder, child: &glib::Object, type_: Option<&str>) {
            let child_widget = child.downcast_ref::<gtk::Widget>();
            match type_ {
                Some("internal") => self.parent_add_child(builder, child, type_),
                Some("header") => self.revealer.set_child(child_widget),
                _ => self.scrolled_window.set_child(child_widget),
            }
        }
    }

    impl WidgetImpl for ScrollingHeaderWidget {}
    impl BoxImpl for ScrollingHeaderWidget {}
}

glib::wrapper! {
    pub struct ScrollingHeaderWidget(ObjectSubclass<imp::ScrollingHeaderWidget>) @extends gtk::Widget, gtk::Box;
}

impl ScrollingHeaderWidget {
    fn set_header_visible(&self, visible: bool) -> bool {
        let widget = self.imp();
        let is_up_to_date = widget.revealer.reveals_child() == visible;
        if !is_up_to_date {
            widget.revealer.set_reveal_child(visible);
        }
        is_up_to_date
    }

    fn is_scrolled_to_top(&self) -> bool {
        self.imp().scrolled_window.vadjustment().value() <= f64::EPSILON
            || self.imp().revealer.reveals_child()
    }

    pub fn connect_header_visibility<F>(&self, f: F)
    where
        F: Fn(bool) + Clone + 'static,
    {
        self.set_header_visible(true);
        f(true);

        let scroll_controller =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll_controller.connect_scroll(
            clone!(@strong f, @weak self as _self => @default-return gtk::Inhibit(false), move |_, _, dy| {
                let visible = dy < 0f64 && _self.is_scrolled_to_top();
                f(visible);
                gtk::Inhibit(!_self.set_header_visible(visible))
            }),
        );

        let swipe_controller = gtk::GestureSwipe::new();
        swipe_controller.set_touch_only(true);
        swipe_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        swipe_controller.connect_swipe(clone!(@weak self as _self => move |_, _, dy| {
            let visible = dy >= 0f64 && _self.is_scrolled_to_top();
            f(visible);
            _self.set_header_visible(visible);
        }));

        self.imp().scrolled_window.add_controller(scroll_controller);
        self.add_controller(swipe_controller);
    }

    pub fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp()
            .scrolled_window
            .connect_edge_reached(move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    f()
                }
            });
    }
}
