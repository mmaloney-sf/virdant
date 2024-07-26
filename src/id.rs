use internment::Intern;
use crate::common::Ident;

pub type Index = usize;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RawId {
    Root,
    Ident(Box<RawId>, Ident),
    Index(Box<RawId>, Index),
    Dup(Box<RawId>),
}

impl Id {
    pub fn root() -> RawId {
        RawId::Root
    }
}

impl RawId {
    pub fn parent(&self) -> RawId {
        match self {
            RawId::Root => panic!("Root RawId has no parent"),
            RawId::Ident(parent, _ident) => *parent.clone(),
            RawId::Index(parent, _index) => *parent.clone(),
            RawId::Dup(parent) => parent.parent(),
        }
    }

    pub fn ident(&self, ident: Ident) -> RawId {
        RawId::Ident(Box::new(self.clone()), ident)
    }

    pub fn index(&self, index: Index) -> RawId {
        RawId::Index(Box::new(self.clone()), index)
    }

    pub fn intern(self) -> Id {
        Id(Intern::new(self))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Id(Intern<RawId>);
