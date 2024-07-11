use crate::common::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unknown,
    Clock,
    Bool,
    Word(Width),
    Vec(Arc<Type>, usize),
    StructType(Ident),
    AltType(Ident),
    Other(String),
}

impl Type {
    pub fn name(&self) -> Ident {
        match self {
            Type::Clock => "Clock".into(),
            Type::Word(width) => format!("Word[{width}]").into(),
            Type::StructType(name) => name.clone(),
            Type::AltType(name) => name.clone(),
            _ => panic!(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Type::Unknown => write!(f, "UNKNOWN"),
            Type::Clock => write!(f, "Clock"),
            Type::Bool => write!(f, "Bool"),
            Type::Word(width) => write!(f, "Word[{width}]"),
            Type::Vec(typ, n) => write!(f, "Vec[{typ}, {n}]"),
            Type::StructType(name) => write!(f, "{name}"),
            Type::AltType(name) => write!(f, "{name}"),
            Type::Other(typename) => write!(f, "{typename}"),
        }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Type::Unknown => write!(f, "UNKNOWN"),
            Type::Clock => write!(f, "Clock"),
            Type::Bool => write!(f, "Bool"),
            Type::Word(width) => write!(f, "Word[{width}]"),
            Type::Vec(typ, n) => write!(f, "Vec[{typ}, {n}]"),
            Type::StructType(name) => write!(f, "{name}"),
            Type::AltType(name) => write!(f, "{name}"),
            Type::Other(typename) => write!(f, "{typename}"),
        }
    }
}
