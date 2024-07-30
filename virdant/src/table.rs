use indexmap::IndexMap;
use std::hash::Hash;
use crate::id::Id;

pub struct Table<E, D>(IndexMap<Id<E>, D>);

impl<E: Copy + Eq + Hash, D: Default> Default for Table<E, D> {
    fn default() -> Self {
        Table::new()
    }
}

impl<E: Copy + Eq + Hash, D: Default> Table<E, D> {
    pub fn new() -> Self {
        Table(IndexMap::new())
    }

    pub fn get(&self, key: Id<E>) -> Option<&D> {
       self.0.get(&key)
    }

    pub fn get_mut(&mut self, key: Id<E>) -> Option<&mut D> {
       self.0.get_mut(&key)
    }

    pub fn is_registered(&self, key: Id<E>) -> bool {
        self.0.contains_key(&key)
    }

    pub fn register(&mut self, key: Id<E>) -> &mut D {
        if self.0.contains_key(&key) {
           self.0.get_mut(&key).unwrap()
        } else {
            self.0.insert(key, D::default());
            &mut self.0[&key]
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &Id<E>> {
        self.0.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Id<E>, &D)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Id<E>, &mut D)> {
        self.0.iter_mut()
    }
}
