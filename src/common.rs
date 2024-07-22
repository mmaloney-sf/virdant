pub use internment::Intern;
pub use std::sync::Arc;
use crate::phase::sourceq::Span;

pub type Val = u64;
pub type Width = u64;
pub type Offset = u64;
pub type Tag = u64;
pub type StaticIndex = u64;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ident(Intern<String>);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct QualIdent(Option<Ident>, Ident);

impl From<Ident> for QualIdent {
    fn from(name: Ident) -> Self {
        QualIdent::new(None, name)
    }
}

impl QualIdent {
    pub fn new(namespace: Option<Ident>, name: Ident) -> Self {
        if let Some(package) = namespace {
            QualIdent(Some(package), name)
        } else {
            QualIdent(None, name)
        }
    }

    pub fn namespace(&self) -> Option<Ident> {
        self.0.clone()
    }

    pub fn name(&self) -> Ident {
        self.1.clone()
    }
}

impl std::borrow::Borrow<str> for Ident {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Path(Intern<String>);

impl std::fmt::Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for QualIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self)
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
    Io(String),
    Other(Option<Span>, String),
    At(Box<VirdantError>, Span),
    Because(Box<VirdantError>, Box<VirdantError>),
    Unknown,
}

#[macro_export]
macro_rules! virdant_error {
    () => {
        VirdantError::Other(None, format!("Unknown Error: in file {} on line {}", file!().to_string(), line!().to_string()))
    };

    ($fmt:literal) => {
        {
            let prelude = format!("Unknown Error: in file {} on line {}: ", file!().to_string(), line!().to_string());
            let msg = format!($fmt);
            VirdantError::Other(None, format!("{prelude}: {msg}"))
        }
    };

    ($fmt:literal, $($arg:expr),*) => {
        {
            let prelude = format!("Unknown Error: in file {} on line {}: ", file!().to_string(), line!().to_string());
            let msg = format!($fmt, $($arg)*); 
            VirdantError::Other(None, format!("{prelude}: {msg}"))
        }
    };
}

#[macro_export]
macro_rules! virdant_error_at {
    ($span:expr) => {
        VirdantError::Other(Some($span), format!("Unknown Error: in file {} on line {}", file!().to_string(), line!().to_string()))
    };

    ($fmt:literal, $span:expr) => {
        {
            let prelude = format!("Unknown Error: in file {} on line {}: ", file!().to_string(), line!().to_string());
            let msg = format!($fmt);
            VirdantError::Other(Some($span), format!("{prelude}: {msg}"))
        }
    };

    ($fmt:literal, $($arg:expr),*, $span:expr) => {
        {
            let prelude = format!("Unknown Error: in file {} on line {}: ", file!().to_string(), line!().to_string());
            let msg = format!($fmt, $($arg)*); 
            VirdantError::Other(Some($span), format!("{prelude}: {msg}"))
        }
    };
}

impl VirdantError {
    pub fn because(self, cause: VirdantError) -> VirdantError {
        VirdantError::Because(Box::new(self), Box::new(cause))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
//    TypeMismatch(Arc<Type>, Arc<Type>),
    CantInfer,
    Other(String),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ErrorReport {
    errors: Vec<VirdantError>,
}

impl From<std::io::Error> for VirdantError {
    fn from(err: std::io::Error) -> VirdantError {
        VirdantError::Io(format!("{err:?}"))
    }
}

impl From<TypeError> for VirdantError {
    fn from(err: TypeError) -> VirdantError {
        VirdantError::TypeError(err)
    }
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

    pub fn add_on_err<T>(&mut self, result: VirdantResult<T>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(err) => {
                self.add(err);
                None
            },
        }
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

impl std::fmt::Display for QualIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let name = self.name();
        if let Some(namespace) = self.namespace() {
            write!(f, "{namespace}::{name}")
        } else {
            write!(f, "{name}")
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl<S> From<S> for Ident where S: Into<String> {
    fn from(s: S) -> Ident {
        Ident(Intern::new(s.into()))
    }
}

impl<S> From<S> for Path where S: Into<String> {
    fn from(s: S) -> Path {
        Path(Intern::new(s.into()))
    }
}

impl From<Ident> for Path {
    fn from(ident: Ident) -> Path {
        Path(Intern::new(ident.0.to_string()))
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

    pub fn parts(&self) -> Vec<Ident> {
        self.0.split('.').map(|id| id.into()).collect()
    }

    pub fn is_local(&self) -> bool {
        self.parts().len() == 1
    }

    pub fn is_nonlocal(&self) -> bool {
        self.parts().len() == 2
    }

    pub fn is_remote(&self) -> bool {
        self.parts().len() > 2
    }

    pub fn as_ident(&self) -> Option<Ident> {
        if self.is_local() {
            Some(self.0.as_ref().clone().into())
        } else {
            None
        }
    }

    pub fn head(&self) -> Ident {
        let parts = self.parts();
        parts[0].clone()
    }

    pub fn tail(&self) -> Option<Path> {
        let parts = self.parts();
        if parts.len() > 1 {
            Some(parts[1..].join(".").into())
        } else {
            None
        }
    }

    pub fn name(&self) -> Ident {
        let parts = self.parts();
        parts[parts.len() - 1].clone()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ExprPath(Vec<usize>);

impl ExprPath {
    pub fn walk(&self) -> &[usize] {
        &self.0
    }
}

impl Default for ExprPath {
    fn default() -> Self {
        ExprPath(vec![])
    }
}

pub fn clog2(n: u64) -> u64 {
    let mut result = 0;
    while n > (1 << result) {
        result += 1;
    }
    result
}

pub fn is_pow2(n: u64) -> bool {
    n != 0 && (n & (n - 1)) == 0
}
