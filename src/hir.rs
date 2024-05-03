mod expr;

use std::collections::HashMap;
use std::sync::Arc;
use crate::common::*;
use crate::ast;
use crate::types::Type;
use crate::ast::ConnectType;

pub use expr::Expr;
pub use expr::ExprNode;
pub use expr::IsExpr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub moddefs: HashMap<Ident, ModDef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModDef {
    pub name: Ident,
    pub components: Vec<Component>,
    pub submodules: Vec<Submodule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submodule {
    pub name: Ident,
    pub moddef: Ident,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component {
    Incoming(Ident, Arc<Type>),
    Outgoing(Ident, Arc<Type>, Expr),
    Wire(Ident, Arc<Type>, Expr),
    Reg(Ident, Arc<Type>, Expr, /*Option<Value>,*/ Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HirType {
    Clock,
    Word(Width),
    Vec(Arc<Type>, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineConnect(pub ConnectType, pub Expr);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect(pub Path, pub ConnectType, pub Expr);

impl InlineConnect {
    pub fn from_ast(connect: &ast::InlineConnect) -> InlineConnect {
        let ast::InlineConnect(connect_type, expr) = connect;
        InlineConnect(*connect_type, Expr::from_ast(expr))
    }
}

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
        }
    }
}
