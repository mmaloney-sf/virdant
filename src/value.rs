use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    X(Type),
    Clock(bool),
    Bool(bool),
    Word(Width, u64),
    Vec(Type, Vec<Value>),
    Struct(Type, Vec<(Field, Value)>),
}

impl Value {
    pub fn type_of(&self) -> Type {
        match self {
            Value::X(typ) => typ.clone(),
            Value::Clock(_b) => Type::Clock,
            Value::Bool(_b) => Type::Bool,
            Value::Word(width, _value) => Type::Word(*width),
            Value::Vec(typ, vs) => Type::Vec(Box::new(typ.clone()), vs.len()),
            Value::Struct(typ, _flds) => typ.clone(),
        }
    }

    pub fn unwrap_word(&self) -> u64 {
        if let Value::Word(_width, value) = self {
            *value
        } else {
            panic!()
        }
    }

    pub fn tick_clock(&mut self) {
        if let Value::Clock(clock) = self {
            *clock = !*clock;
        } else {
            panic!();
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::X(_typ) => write!(f, "XXX"),
            Value::Clock(_typ) => write!(f, "(clock)"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Word(w, n) => write!(f, "{n}w{w}"),
            Value::Vec(_typ, vs) => {
                write!(f, "[")?;
                for (i, v) in vs.iter().enumerate() {
                    if i + 1 < vs.len() {
                        write!(f, "{v}, ")?;
                    } else {
                        write!(f, "{v}")?;
                    }
                }
                write!(f, "]")
            },
            Value::Struct(typ, fields) => {
                write!(f, "struct {typ} {{ ")?;
                for (i, (fld, v)) in fields.iter().enumerate() {
                    write!(f, "{fld} = {v}")?;
                    if i + 1 < fields.len() {
                        write!(f, ", ")?;
                    } else {
                        write!(f, " ")?;
                    }
                }
                write!(f, "}}")
            },
//            Value::Enum(typedef, name) => write!(f, "{}::{}", typedef.name(), name),
//            Value::Ctor(ctor, vs) => {
//                write!(f, "@{ctor}")?;
//                if vs.len() > 0 {
//                    write!(f, "(")?;
//                    for (i, v) in vs.iter().enumerate() {
//                        write!(f, "{v:?}")?;
//                        if i + 1 < vs.len() {
//                            write!(f, ", ")?;
//                        }
//                    }
//                    write!(f, ")")
//                } else {
//                    Ok(())
//                }
//            },
        }
    }
}


