pub use std::sync::Arc;

pub type Width = u64;
pub type Field = String;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ident(String);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Path(String);

impl std::fmt::Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

pub type VirdantResult<T> = Result<T, VirdantError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirdantError {
    Multiple(Vec<VirdantError>),
    TypeError(TypeError),
    ParseError(String),
    Other(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    TypeMismatch(),
    CantInfer,
    Other,
}

#[derive(Debug, Clone)]
pub struct ErrorReport {
    errors: Vec<VirdantError>,
}

impl ErrorReport {
    pub fn new() -> ErrorReport {
        ErrorReport {
            errors: vec![],
        }
    }

    pub fn add<E: Into<VirdantError>>(&mut self, error: E) {
        self.errors.push(error.into());
    }

    pub fn check(self) -> Result<(), VirdantError> {
        if self.errors.len() == 0 {
            Ok(())
        } else if self.errors.len() == 1 {
            Err(self.errors[0].clone())
        } else {
            Err(VirdantError::Multiple(self.errors))
        }
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl<S> From<S> for Ident where S: Into<String> {
    fn from(s: S) -> Ident {
        Ident(s.into())
    }
}

impl<S> From<S> for Path where S: Into<String> {
    fn from(s: S) -> Path {
        Path(s.into())
    }
}

impl Ident {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_path(&self) -> Path {
        self.0.as_str().into()
    }
}

impl Path {
    pub fn join(&self, other: &Path) -> Path {
        format!("{}.{}", self.0, other.0).into()
    }

    pub fn parent(&self) -> Path {
        let parts = self.parts();
        parts[0..parts.len()-1].join(".").into()
    }

    pub fn parts(&self) -> Vec<&str> {
        self.0.split('.').collect()
    }

    pub fn is_local(&self) -> bool {
        self.parts().len() == 1
    }

    pub fn is_foreign(&self) -> bool {
        self.parts().len() == 2
    }

    pub fn is_remote(&self) -> bool {
        self.parts().len() > 2
    }

    pub fn as_ident(&self) -> Option<Ident> {
        if self.is_local() {
            Some(self.0.clone().into())
        } else {
            None
        }
    }
}
