use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct Clock {
    interval_ms: u32,
    source: Cell<Option<glib::source::SourceId>>,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            interval_ms: 1000,
            source: Cell::new(None),
        }
    }

    pub fn start<F: Fn() + 'static>(&self, tick: F) {
        let new_source = Some(glib::timeout_add_local(self.interval_ms, move || {
            tick();
            glib::Continue(true)
        }));
        if let Some(previous_source) = self.source.replace(new_source) {
            glib::source_remove(previous_source);
        }
    }

    pub fn stop(&self) {
        let new_source = None;
        if let Some(previous_source) = self.source.replace(new_source) {
            glib::source_remove(previous_source);
        }
    }
}

#[derive(Clone)]
pub struct Debouncer(Rc<Cell<Option<glib::source::SourceId>>>);

impl Debouncer {
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(None)))
    }

    pub fn debounce<F: Fn() + 'static>(&self, interval_ms: u32, f: F) {
        let source_clone = Rc::downgrade(&self.0);
        let new_source = glib::timeout_add_local(interval_ms, move || {
            f();
            if let Some(cell) = source_clone.upgrade() {
                cell.set(None);
            }
            glib::Continue(false)
        });
        if let Some(previous_source) = self.0.replace(Some(new_source)) {
            glib::source_remove(previous_source);
        }
    }
}

pub struct Animator<EasingFn> {
    progress: Rc<Cell<u16>>,
    ease_fn: EasingFn,
}

pub type AnimatorDefault = Animator<fn(f64) -> f64>;

impl AnimatorDefault {
    fn ease_in_out(x: f64) -> f64 {
        match x {
            x if x < 0.5 => 0.5 * 2f64.powf(20.0 * x - 10.0),
            _ => 0.5 * (2.0 - 2f64.powf(-20.0 * x + 10.0)),
        }
    }

    pub fn ease_in_out_animator() -> AnimatorDefault {
        Animator {
            progress: Rc::new(Cell::new(0)),
            ease_fn: Self::ease_in_out,
        }
    }
}

impl<EasingFn> Animator<EasingFn>
where
    EasingFn: 'static + Copy + Fn(f64) -> f64,
{
    pub fn animate<F: Fn(f64) -> bool + 'static>(&self, steps: u16, f: F) {
        self.progress.set(0);
        let ease_fn = self.ease_fn;

        let progress = Rc::downgrade(&self.progress);
        glib::timeout_add_local(16, move || {
            let mut continue_ = false;
            if let Some(progress) = progress.upgrade() {
                let step = progress.get();
                continue_ = step < steps;
                if continue_ {
                    progress.set(step + 1);
                    let p = ease_fn(step as f64 / steps as f64);
                    continue_ = f(p);
                }
            }
            glib::Continue(continue_)
        });
    }
}

pub fn wrap_flowbox_item<
    Model: glib::IsA<glib::Object>,
    Widget: glib::IsA<gtk::Widget>,
    F: Fn(&Model) -> Widget,
>(
    item: &glib::Object,
    f: F,
) -> gtk::Widget {
    let item = item.downcast_ref::<Model>().unwrap();
    let widget = f(item);
    let child = gtk::FlowBoxChild::new();
    child.add(&widget);
    child.show_all();
    child.upcast::<gtk::Widget>()
}

pub fn format_duration(duration: f64) -> String {
    let seconds = (duration / 1000.0) as i32;
    let minutes = seconds.div_euclid(60);
    let seconds = seconds.rem_euclid(60);
    format!("{}:{:02}", minutes, seconds)
}

fn parent_scrolled_window(widget: &gtk::Widget) -> Option<gtk::ScrolledWindow> {
    let parent = widget.get_parent()?;
    match parent.downcast_ref::<gtk::ScrolledWindow>() {
        Some(scrolled_window) => Some(scrolled_window.clone()),
        None => parent_scrolled_window(&parent),
    }
}

pub fn in_viewport(widget: &gtk::Widget) -> Option<bool> {
    let window = parent_scrolled_window(widget)?;
    let adjustment = window.get_vadjustment()?;
    let (_, y) = widget.translate_coordinates(&window, 0, 0)?;
    let y = y as f64;
    Some(y > 0.0 && y < 0.9 * adjustment.get_page_size())
}

pub fn vscroll_to(widget: &gtk::Widget, progress: f64) -> Option<f64> {
    let window = parent_scrolled_window(widget)?;
    let adjustment = window.get_vadjustment()?;
    let (_, y) = widget.translate_coordinates(&window, 0, 0)?;
    let y = y as f64;
    let target = adjustment.get_value() + y * progress;
    adjustment.set_value(target);
    Some(target)
}
