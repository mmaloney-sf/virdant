mod expr;

use std::collections::HashMap;
use std::sync::Arc;
use crate::common::*;
use crate::ast;
use crate::types::Type;
use crate::ast::ConnectType;
use crate::elab;

pub use expr::Expr;
pub use expr::ExprNode;
pub use expr::IsExpr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub moddefs: HashMap<Ident, Arc<ModDef>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModDef {
    pub name: Ident,
    pub components: Vec<Component>,
    pub submodules: Vec<Submodule>,
    pub connects: Vec<Connect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submodule {
    pub name: Ident,
    pub moddef: Ident,
//    pub incoming_port_connects: Vec<Connect>, // TODO
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component {
    Incoming(Ident, Arc<Type>),
    Outgoing(Ident, Arc<Type>, Expr),
    Wire(Ident, Arc<Type>, Expr),
    Reg(Ident, Arc<Type>, Path, /*Option<Value>,*/ Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineConnect(pub ConnectType, pub Expr);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect(pub Path, pub ConnectType, pub Expr);

impl Component {
    pub fn name(&self) -> Ident {
        match self {
            Component::Incoming(name, _typ) => name.clone(),
            Component::Outgoing(name, _typ, _connect) => name.clone(),
            Component::Wire(name, _typ, _connect) => name.clone(),
            Component::Reg(name, _typ, _clk, /*Option<Value>,*/ _connect) => name.clone(),
        }
    }

    pub fn type_of(&self) -> Arc<Type> {
        match self {
            Component::Incoming(_name, typ) => typ.clone(),
            Component::Outgoing(_name, typ, _connect) => typ.clone(),
            Component::Wire(_name, typ, _connect) => typ.clone(),
            Component::Reg(_name, typ, _clk, /*Option<Value>,*/ _connect) => typ.clone(),
        }
    }
}

impl Type {
    pub fn from_ast(typ: &ast::Type) -> Arc<Type> {
        match typ {
            ast::Type::Clock => Type::Clock.into(),
            ast::Type::Word(width) => Type::Word(*width).into(),
            ast::Type::Vec(inner, len) => Type::Vec(Type::from_ast(inner), *len).into(),
            ast::Type::TypeRef(name) => Type::TypeRef(name.clone()).into(),
        }
    }
}

impl Package {
    pub fn elab(&self, top: Ident) -> VirdantResult<elab::Elab> {
        let moddef = self.moddefs.get(&top).ok_or(VirdantError::Other("Unknown module".into()))?.clone();
        self.elab_moddef(moddef)
    }

    fn elab_moddef(&self, moddef: Arc<ModDef>) -> VirdantResult<elab::Elab> {
        let mut submodules = HashMap::new();

        for submodule in &moddef.submodules {
            let submodule_moddef = self.moddefs.get(&submodule.moddef).ok_or(VirdantError::Other("Unknown module".into()))?.clone();
            let submodule_elab = self.elab_moddef(submodule_moddef)?;
            submodules.insert(submodule.name.clone(), submodule_elab);
        }

        Ok(elab::Elab {
            moddef,
            submodules,
        })
    }
}

impl ModDef {
    pub fn nonlocal_connects_to(&self, submodule: Ident) -> HashMap<Ident, Expr> {
        let mut result = HashMap::new();
        for Connect(target, _connect_type, expr) in &self.connects {
            let port: Ident = target.parts()[1].into();
            if target.parent().as_ident().unwrap() == submodule {
                result.insert(port, expr.clone());
            }
        }
        result
    }
}
