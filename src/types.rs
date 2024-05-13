use crate::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unknown,
    Clock,
    Bool,
    Word(Width),
    Vec(Arc<Type>, usize),
    TypeRef(Ident),
    Other(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Type::Unknown => write!(f, "UNKNOWN"),
            Type::Clock => write!(f, "Clock"),
            Type::Bool => write!(f, "Bool"),
            Type::Word(width) => write!(f, "Word[{width}]"),
            Type::Vec(typ, n) => write!(f, "Vec[{typ}, {n}]"),
            Type::TypeRef(name) => write!(f, "{name}"),
            Type::Other(typename) => write!(f, "{typename}"),
        }
    }
}
