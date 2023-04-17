use gio::prelude::*;
use gio::ListModel;
use glib::Properties;
use glib::StaticType;
use gtk::subclass::prelude::*;
use std::cell::{Cell, Ref, RefCell, RefMut};

use super::support::*;
use crate::app::models::*;

#[must_use]
pub struct SongListModelPending<'a> {
    change: Option<ListRangeUpdate>,
    song_list_model: &'a mut SongListModel,
}

impl<'a> SongListModelPending<'a> {
    fn new(change: Option<ListRangeUpdate>, song_list_model: &'a mut SongListModel) -> Self {
        Self {
            change,
            song_list_model,
        }
    }

    pub fn and<Op>(self, op: Op) -> Self
    where
        Op: FnOnce(&mut SongListModel) -> SongListModelPending<'_> + 'static,
    {
        let Self {
            change,
            song_list_model,
        } = self;

        let new_change = op(song_list_model).change;

        let merged_change = if let (Some(change), Some(new_change)) = (change, new_change) {
            Some(change.merge(new_change))
        } else {
            change.or(new_change)
        };

        Self {
            change: merged_change,
            song_list_model,
        }
    }

    pub fn commit(self) -> bool {
        let Self {
            change,
            song_list_model,
        } = self;
        song_list_model.notify_changes(change);
        change.is_some()
    }
}

glib::wrapper! {
    pub struct SongListModel(ObjectSubclass<imp::SongListModel>) @implements gio::ListModel;
}

impl SongListModel {
    pub fn new(batch_size: u32) -> Self {
        glib::Object::builder()
            .property("batch-size", batch_size)
            .build()
    }

    fn inner_mut(&mut self) -> RefMut<SongList> {
        self.imp().get_mut()
    }

    fn inner(&self) -> Ref<SongList> {
        self.imp().get()
    }

    fn notify_changes(&self, changes: impl IntoIterator<Item = ListRangeUpdate> + 'static) {
        if cfg!(not(test)) {
            glib::source::idle_add_local_once(clone!(@weak self as s => move || {
                for ListRangeUpdate(a, b, c) in changes.into_iter() {
                    debug!("pos {}, removed {}, added {}", a, b, c);
                    s.items_changed(a as u32, b as u32, c as u32);
                }
            }));
        }
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: Fn(usize, &SongModel),
    {
        for (i, song) in self.inner().iter().enumerate() {
            f(i, song);
        }
    }

    pub fn collect(&self) -> Vec<SongDescription> {
        self.inner().iter().map(|s| s.into_description()).collect()
    }

    pub fn map_collect<T>(&self, map: impl Fn(SongDescription) -> T) -> Vec<T> {
        self.inner()
            .iter()
            .map(|s| map(s.into_description()))
            .collect()
    }

    pub fn add(&mut self, song_batch: SongBatch) -> SongListModelPending {
        let range = self.inner_mut().add(song_batch);
        SongListModelPending::new(range, self)
    }

    pub fn get(&self, id: &str) -> Option<SongModel> {
        self.inner().get(id).cloned()
    }

    pub fn index(&self, i: usize) -> Option<SongModel> {
        self.inner().index(i).cloned()
    }

    pub fn index_continuous(&self, i: usize) -> Option<SongModel> {
        self.inner().index_continuous(i).cloned()
    }

    pub fn song_batch_for(&self, i: usize) -> Option<SongBatch> {
        self.inner().song_batch_for(i)
    }

    pub fn last_batch(&self) -> Option<Batch> {
        self.inner().last_batch()
    }

    pub fn needed_batch_for(&self, i: usize) -> Option<Batch> {
        self.inner().needed_batch_for(i)
    }

    pub fn partial_len(&self) -> usize {
        self.inner().partial_len()
    }

    pub fn len(&self) -> usize {
        self.inner().len()
    }

    pub fn append(&mut self, songs: Vec<SongDescription>) -> SongListModelPending {
        let range = self.inner_mut().append(songs);
        SongListModelPending::new(Some(range), self)
    }

    pub fn prepend(&mut self, songs: Vec<SongDescription>) -> SongListModelPending {
        let range = self.inner_mut().prepend(songs);
        SongListModelPending::new(Some(range), self)
    }

    pub fn find_index(&self, song_id: &str) -> Option<usize> {
        self.inner().find_index(song_id)
    }

    pub fn remove(&mut self, ids: &[String]) -> SongListModelPending {
        let change = self.inner_mut().remove(ids);
        SongListModelPending::new(Some(change), self)
    }

    pub fn move_down(&mut self, a: usize) -> SongListModelPending {
        let swap = self.inner_mut().swap(a + 1, a);
        SongListModelPending::new(swap, self)
    }

    pub fn move_up(&mut self, a: usize) -> SongListModelPending {
        let swap = self.inner_mut().swap(a - 1, a);
        SongListModelPending::new(swap, self)
    }

    pub fn clear(&mut self) -> SongListModelPending {
        let removed = self.inner_mut().clear();
        SongListModelPending::new(Some(removed), self)
    }
}

mod imp {

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::SongListModel)]
    pub struct SongListModel {
        #[property(get, set = Self::set_batch_size, name = "batch-size")]
        batch_size: Cell<u32>,
        song_list: RefCell<Option<SongList>>,
    }

    impl SongListModel {
        fn set_batch_size(&self, batch_size: u32) {
            self.batch_size.set(batch_size);
            self.song_list
                .replace(Some(SongList::new_sized(batch_size as usize)));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongListModel {
        const NAME: &'static str = "SongList";
        type Type = super::SongListModel;
        type ParentType = glib::Object;
        type Interfaces = (ListModel,);
    }

    impl ObjectImpl for SongListModel {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }

    impl ListModelImpl for SongListModel {
        fn item_type(&self) -> glib::Type {
            SongModel::static_type()
        }

        fn n_items(&self) -> u32 {
            self.get().partial_len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.get()
                .index_continuous(position as usize)
                .map(|m| m.clone().upcast())
        }
    }

    impl SongListModel {
        pub fn get_mut(&self) -> RefMut<SongList> {
            RefMut::map(self.song_list.borrow_mut(), |s| {
                s.as_mut().expect("set at construction")
            })
        }

        pub fn get(&self) -> Ref<SongList> {
            Ref::map(self.song_list.borrow(), |s| {
                s.as_ref().expect("set at construction")
            })
        }
    }
}
