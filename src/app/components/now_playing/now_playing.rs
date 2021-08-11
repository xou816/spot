use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use crate::app::components::{screen_add_css_provider, Component, EventListener, Playlist};
use crate::app::{state::PlaybackEvent, AppEvent};

use super::NowPlayingModel;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/alextren/Spot/components/now_playing.ui")]
    pub struct NowPlayingWidget {
        #[template_child]
        pub listbox: TemplateChild<gtk::ListBox>,

        #[template_child]
        pub shuffle: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NowPlayingWidget {
        const NAME: &'static str = "NowPlayingWidget";
        type Type = super::NowPlayingWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NowPlayingWidget {}
    impl WidgetImpl for NowPlayingWidget {}
    impl BoxImpl for NowPlayingWidget {}
}

glib::wrapper! {
    pub struct NowPlayingWidget(ObjectSubclass<imp::NowPlayingWidget>) @extends gtk::Widget, gtk::Box;
}

impl NowPlayingWidget {
    fn new() -> Self {
        screen_add_css_provider(resource!("/components/now_playing.css"));
        glib::Object::new(&[]).expect("Failed to create an instance of NowPlayingWidget")
    }

    fn songlist_widget(&self) -> &gtk::ListBox {
        imp::NowPlayingWidget::from_instance(self).listbox.as_ref()
    }

    fn connect_shuffle<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        imp::NowPlayingWidget::from_instance(self)
            .shuffle
            .connect_clicked(move |_| f());
    }
}

pub struct NowPlaying {
    widget: NowPlayingWidget,
    model: Rc<NowPlayingModel>,
    children: Vec<Box<dyn EventListener>>,
}

impl NowPlaying {
    pub fn new(model: Rc<NowPlayingModel>) -> Self {
        let widget = NowPlayingWidget::new();

        widget.connect_shuffle(clone!(@weak model => move || {
            model.toggle_shuffle();
        }));

        let playlist = Playlist::new(widget.songlist_widget().clone(), model.clone());

        Self {
            widget,
            model,
            children: vec![Box::new(playlist)],
        }
    }
}

impl Component for NowPlaying {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.widget.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for NowPlaying {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) = event {
            self.model.load_more_if_needed();
        }
        self.broadcast_event(event);
    }
}
