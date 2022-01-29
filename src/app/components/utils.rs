use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone)]
pub struct Clock {
    interval_ms: u32,
    source: Rc<Cell<Option<glib::source::SourceId>>>,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl std::fmt::Debug for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clock")
            .field("interval_ms", &self.interval_ms)
            .finish()
    }
}

impl Clock {
    pub fn new(interval_ms: u32) -> Self {
        Self {
            interval_ms,
            source: Rc::new(Cell::new(None)),
        }
    }

    pub fn start<F: Fn() + 'static>(&self, tick: F) {
        let new_source = Some(glib::timeout_add_local(
            Duration::from_millis(self.interval_ms.into()),
            move || {
                tick();
                glib::Continue(true)
            },
        ));
        if let Some(previous_source) = self.source.replace(new_source) {
            previous_source.remove();
        }
    }

    pub fn stop(&self) {
        let new_source = None;
        if let Some(previous_source) = self.source.replace(new_source) {
            previous_source.remove();
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
        let new_source =
            glib::timeout_add_local(Duration::from_millis(interval_ms.into()), move || {
                f();
                if let Some(cell) = source_clone.upgrade() {
                    cell.set(None);
                }
                glib::Continue(false)
            });
        if let Some(previous_source) = self.0.replace(Some(new_source)) {
            previous_source.remove();
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
        glib::timeout_add_local(Duration::from_millis(16), move || {
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
    Widget: gtk::glib::IsA<gtk::Widget>,
    F: Fn(&Model) -> Widget,
>(
    item: &glib::Object,
    f: F,
) -> gtk::Widget {
    let item = item.downcast_ref::<Model>().unwrap();
    let widget = f(item);
    let child = gtk::FlowBoxChild::new();
    child.set_child(Some(&widget));
    child.upcast::<gtk::Widget>()
}

pub fn format_duration(duration: f64) -> String {
    let seconds = (duration / 1000.0) as i32;
    let hours = seconds.div_euclid(3600);
    let minutes = seconds.div_euclid(60).rem_euclid(60);
    let seconds = seconds.rem_euclid(60);
    if hours > 0 {
        format!("{}∶{:02}∶{:02}", hours, minutes, seconds)
    } else {
        format!("{}∶{:02}", minutes, seconds)
    }
}
