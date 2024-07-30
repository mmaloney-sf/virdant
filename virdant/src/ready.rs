use indexmap::IndexSet;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct Ready<T>(IndexSet<T>);

#[derive(Clone, Debug)]
pub enum ReadyErr {
    NotSet,
    MultiplySet,
}

impl<T: Clone + Hash + Eq> Default for Ready<T> {
    fn default() -> Self {
        Ready(IndexSet::new())
    }
}

impl<T: Clone + Hash + Eq> Ready<T> {
    pub fn new() -> Self {
        Ready(IndexSet::new())
    }

    pub fn set(&mut self, t: T) {
        self.0.insert(t);
    }

    pub fn get(&self) -> Result<&T, ReadyErr> {
        if self.0.len() == 1 {
            Ok(&self.0[0])
        } else if self.0.len() == 0 {
            Err(ReadyErr::NotSet)
        } else {
            Err(ReadyErr::MultiplySet)
        }
    }
}
