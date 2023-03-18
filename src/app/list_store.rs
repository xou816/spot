use gio::prelude::*;
use glib::clone::{Downgrade, Upgrade};
use std::iter::Iterator;
use std::marker::PhantomData;

pub struct ListStore<GType> {
    store: gio::ListStore,
    _marker: PhantomData<GType>,
}

pub struct WeakListStore<GType> {
    store: <gio::ListStore as Downgrade>::Weak,
    _marker: PhantomData<GType>,
}

impl<GType> ListStore<GType>
where
    GType: IsA<glib::Object>,
{
    pub fn new() -> Self {
        Self {
            store: gio::ListStore::new(GType::static_type()),
            _marker: PhantomData,
        }
    }

    pub fn unsafe_store(&self) -> &gio::ListStore {
        &self.store
    }

    pub fn prepend(&mut self, elements: impl Iterator<Item = GType>) {
        let upcast_vec: Vec<glib::Object> = elements.map(|e| e.upcast::<glib::Object>()).collect();
        self.store.splice(0, 0, &upcast_vec[..]);
    }

    pub fn extend(&mut self, elements: impl Iterator<Item = GType>) {
        let upcast_vec: Vec<glib::Object> = elements.map(|e| e.upcast::<glib::Object>()).collect();
        self.store.splice(self.store.n_items(), 0, &upcast_vec[..]);
    }

    pub fn replace_all(&mut self, elements: impl Iterator<Item = GType>) {
        let upcast_vec: Vec<glib::Object> = elements.map(|e| e.upcast::<glib::Object>()).collect();
        self.store.splice(0, self.store.n_items(), &upcast_vec[..]);
    }

    pub fn insert(&mut self, position: u32, element: GType) {
        self.store.insert(position, &element);
    }

    pub fn remove(&mut self, position: u32) {
        self.store.remove(position);
    }

    pub fn get(&self, index: u32) -> GType {
        self.store.item(index).unwrap().downcast::<GType>().unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = GType> + '_ {
        let store = &self.store;
        let count = store.n_items();
        (0..count).map(move |i| self.get(i))
    }

    pub fn len(&self) -> usize {
        self.store.n_items() as usize
    }

    pub fn eq<F, O>(&self, other: &[O], comparison: F) -> bool
    where
        F: Fn(&GType, &O) -> bool,
    {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(left, right)| comparison(&left, right))
    }
}

impl<GType> Clone for ListStore<GType> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            _marker: PhantomData,
        }
    }
}

impl<GType> Downgrade for ListStore<GType> {
    type Weak = WeakListStore<GType>;

    fn downgrade(&self) -> Self::Weak {
        Self::Weak {
            store: Downgrade::downgrade(&self.store),
            _marker: PhantomData,
        }
    }
}

impl<GType> Upgrade for WeakListStore<GType> {
    type Strong = ListStore<GType>;

    fn upgrade(&self) -> Option<Self::Strong> {
        Some(Self::Strong {
            store: self.store.upgrade()?,
            _marker: PhantomData,
        })
    }
}
