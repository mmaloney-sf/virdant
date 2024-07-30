//! Defines the [`VirErr`] and [`VirErrs`] types.

use crate::parse::ParseError;
use crate::id::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirErr {
    Io(String),
    Parse(ParseError),
    DupItem(Id<Item>),
    CantImport(Id<Package>),
    DupImport(Id<Package>),
    Other(String),
}

#[derive(Debug, Clone, Default)]
pub struct VirErrs {
    errors: Vec<VirErr>,
}

impl From<std::io::Error> for VirErr {
    fn from(err: std::io::Error) -> VirErr {
        VirErr::Io(format!("{err:?}"))
    }
}

impl VirErrs {
    pub fn new() -> VirErrs {
        VirErrs {
            errors: vec![],
        }
    }

    pub fn add<E: Into<VirErr>>(&mut self, error: E) {
        self.errors.push(error.into());
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
}

impl std::ops::Index<usize> for VirErrs {
    type Output = VirErr;

    fn index(&self, index: usize) -> &Self::Output {
        &self.errors[index]
    }
}
