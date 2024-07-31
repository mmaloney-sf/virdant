use crate::id::*;

pub type Nat = u64;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type(TypeScheme, Option<Nat>);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TypeScheme {
    StructDef(Id<StructDef>),
    UnionDef(Id<UnionDef>),
    BuiltinDef(Id<BuiltinDef>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CtorSig(Id<Ctor>, Vec<(String, Type)>, Type);

impl CtorSig {
    pub fn new(ctor: Id<Ctor>, params: Vec<(String, Type)>, ret_typ: Type) -> Self {
        CtorSig(ctor, params, ret_typ)
    }

    pub fn ctor(&self) -> Id<Ctor> {
        self.0
    }

    pub fn params(&self) -> &[(String, Type)] {
        &self.1
    }

    pub fn ret(&self) -> Type {
        self.2
    }
}

impl Type {
    pub fn structdef(structdef: Id<StructDef>) -> Self {
        Type(TypeScheme::StructDef(structdef), None)
    }

    pub fn uniondef(uniondef: Id<UnionDef>) -> Self {
        Type(TypeScheme::UnionDef(uniondef), None)
    }

    pub fn builtindef(builtindef: Id<BuiltinDef>, arg: Option<Nat>) -> Self {
        Type(TypeScheme::BuiltinDef(builtindef), arg)
    }

    fn itemdef(&self) -> Id<Item> {
        match self.0 {
            TypeScheme::StructDef(structdef) => structdef.as_item(),
            TypeScheme::UnionDef(uniondef) => uniondef.as_item(),
            TypeScheme::BuiltinDef(builtindef) => builtindef.as_item(),
        }
    }

    fn args(&self) -> Option<Nat> {
        self.1.clone()
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item_def = self.itemdef();
        let arg_str = match self.args() {
            None => format!(""),
            Some(n) => format!("[{n}]"),
        };
        write!(f, "{item_def}{arg_str}")
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl std::fmt::Display for CtorSig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.ctor();
        write!(f, "{name}(")?;
        for (i, (name, typ)) in self.params().iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name} : {typ}")?;
        }
        write!(f, ") : {}", self.ret())?;
        Ok(())
    }
}

impl std::fmt::Debug for CtorSig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
