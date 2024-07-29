use internment::Intern;
use std::marker::PhantomData;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Id<T>(Intern<String>, PhantomData<T>);


impl<T> Id<T> {
    pub fn new<S: Into<String>>(s: S) -> Id<T> {
        Id(Intern::new(s.into()), PhantomData::default())
    }
}

impl<'a, T> From<&'a str> for Id<T> {
    fn from(id: &'a str) -> Self {
        Id(Intern::new(id.to_string()), PhantomData::default())
    }
}

impl<T> From<String> for Id<T> {
    fn from(id: String) -> Self {
        Id(Intern::new(id), PhantomData::default())
    }
}

impl<T> From<Id<T>> for Intern<String> {
    fn from(value: Id<T>) -> Self {
        value.0
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub mod types {
    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub struct Package;

    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub struct Item;
}

pub use types::*;
