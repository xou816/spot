use std::marker::PhantomData;
use std::iter::Iterator;
use std::convert::Into;
use glib::prelude::*;
use gio::prelude::*;

pub struct ListStore<T, GType> {
    store: gio::ListStore,
    vec: Vec<T>,
    _marker: PhantomData<GType>
}

impl <T, GType> ListStore<T, GType>
    where T: Clone + Into<GType>, GType: IsA<glib::Object> {

    pub fn new() -> Self {
        Self {
            store: gio::ListStore::new(GType::static_type()),
            vec: vec![],
            _marker: PhantomData
        }
    }

    pub fn unsafe_store(&self) -> &gio::ListStore {
        &self.store
    }

    pub fn remove_all(&mut self) {
        self.vec.clear();
        self.store.remove_all();
    }

    pub fn append(&mut self, element: T) {
        let vec = &mut self.vec;
        vec.push(element.clone());
        self.store.append(&element.into());
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.vec.iter()
    }
}

impl <T, GType> ListStore<T, GType>
    where T: Eq {

    pub fn eq(&self, other: &Vec<T>) -> bool {
        self.vec == *other
    }
}

impl <T, GType> Clone for ListStore<T, GType> where T: Clone {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            vec: self.vec.clone(),
            _marker: PhantomData
        }
    }
}
