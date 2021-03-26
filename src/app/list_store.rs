use gio::prelude::*;
use gio::ListModelExt;
use std::iter::Iterator;
use std::marker::PhantomData;

pub enum ListDiff<GType> {
    Set(Vec<GType>),
    Append(Vec<GType>),
}

pub struct ListStore<GType> {
    store: gio::ListStore,
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

    pub fn update(&mut self, diff: ListDiff<GType>) {
        match diff {
            ListDiff::Set(elements) => self.replace_all(elements.into_iter()),
            ListDiff::Append(elements) => self.extend(elements.into_iter()),
        }
    }

    pub fn unsafe_store(&self) -> &gio::ListStore {
        &self.store
    }

    pub fn extend(&mut self, elements: impl Iterator<Item = GType>) {
        let upcast_vec: Vec<glib::Object> = elements.map(|e| e.upcast::<glib::Object>()).collect();
        self.store
            .splice(self.store.get_n_items(), 0, &upcast_vec[..]);
    }

    pub fn replace_all(&mut self, elements: impl Iterator<Item = GType>) {
        let upcast_vec: Vec<glib::Object> = elements.map(|e| e.upcast::<glib::Object>()).collect();
        self.store
            .splice(0, self.store.get_n_items(), &upcast_vec[..]);
    }

    pub fn insert(&mut self, position: u32, element: GType) {
        self.store.insert(position, &element);
    }

    pub fn remove(&mut self, position: u32) {
        self.store.remove(position);
    }

    pub fn get(&self, index: u32) -> GType {
        self.store
            .get_object(index)
            .unwrap()
            .downcast::<GType>()
            .unwrap()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = GType> + 'a {
        let store = &self.store;
        let count = store.get_n_items();
        (0..count).into_iter().map(move |i| self.get(i))
    }

    pub fn len(&self) -> usize {
        self.store.get_n_items() as usize
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
