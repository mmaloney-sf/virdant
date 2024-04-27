use crate::common::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unknown,
    Clock,
    Bool,
    Word(Width),
    Vec(Box<Type>, usize),
    Other(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Type::Unknown => write!(f, "UNKNOWN"),
            Type::Clock => write!(f, "Clock"),
            Type::Bool => write!(f, "Bool"),
            Type::Word(width) => write!(f, "Word<{width}>"),
            Type::Vec(typ, n) => write!(f, "Vec<{typ}, {n}>"),
            Type::Other(typename) => write!(f, "{typename}"),
        }
    }
}
