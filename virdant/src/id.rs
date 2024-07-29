//! Defines the [`Id<T>`](Id) type.
//!
//! The [`Id<T>`](Id) type is a wrapper around an [`Intern<String>`](internment::Intern)
//! for use in [`HashMap`](std::collections::HashMap)'s in the [`Virdant`](crate::Virdant) struct.

use internment::Intern;
use std::marker::PhantomData;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Id<T>(Intern<String>, PhantomData<T>);


impl<T> Id<T> {
    pub fn new<S: Into<String>>(s: S) -> Id<T> {
        Id(Intern::new(s.into()), PhantomData)
    }

    pub fn cast<S>(&self) -> Id<S> {
        Id(self.0, PhantomData)
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
    macro_rules! id_type {
        ($name:ident) => {
            #[derive(Clone, Copy, Eq, PartialEq, Hash)]
            pub struct $name(());
        };
    }

    id_type!(Package);

    id_type!(Item);

    id_type!(ModDef);
    id_type!(UnionDef);
    id_type!(StructDef);
    id_type!(PortDef);

    id_type!(Ctor);
    id_type!(Field);
}

pub use types::*;

impl Id<ModDef> {
    pub fn as_item(&self) -> Id<Item> {
        self.cast()
    }
}

impl Id<UnionDef> {
    pub fn as_item(&self) -> Id<Item> {
        self.cast()
    }
}

impl Id<StructDef> {
    pub fn as_item(&self) -> Id<Item> {
        self.cast()
    }
}

impl Id<PortDef> {
    pub fn as_item(&self) -> Id<Item> {
        self.cast()
    }
}
