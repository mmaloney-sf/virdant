//! Defines the [`VirErr`] and [`VirErrs`] types.

use indexmap::IndexSet;
use std::hash::Hash;

use crate::parse::ParseError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VirErr {
    Io(String),
    Parse(ParseError),
    DupItem(String),
    CantImport(String),
    DupImport(String),
    UnresolvedIdent(String),
    ItemDepCycle(Vec<String>),
    KindError(String),
    Other(String),
}

#[derive(Debug, Clone, Default)]
pub struct VirErrs {
    errors: IndexSet<VirErr>,
}

impl From<std::io::Error> for VirErr {
    fn from(err: std::io::Error) -> VirErr {
        VirErr::Io(format!("{err:?}"))
    }
}

impl VirErrs {
    pub fn new() -> VirErrs {
        VirErrs {
            errors: IndexSet::new(),
        }
    }

    pub fn add<E: Into<VirErr>>(&mut self, error: E) {
        self.errors.insert(error.into());
    }

    pub fn add_on_err<T>(&mut self, result: Result<T, VirErr>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(err) => {
                self.add(err);
                None
            },
        }
    }

    pub fn check(self) -> Result<(), VirErrs> {
        if self.errors.len() == 0 {
            Ok(())
        } else {
            Err(self)
        } 
    }

    pub fn extend(&mut self, others: VirErrs) {
        self.errors.extend(others.errors);
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn into_iter(self) -> impl Iterator<Item = VirErr> {
        self.errors.into_iter()
    }
}

impl std::ops::Index<usize> for VirErrs {
    type Output = VirErr;

    fn index(&self, index: usize) -> &Self::Output {
        &self.errors[index]
    }
}
