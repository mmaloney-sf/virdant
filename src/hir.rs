mod expr;

use std::collections::HashMap;
use std::sync::Arc;
use crate::common::*;
use crate::ast;
use crate::types::Type;
use crate::value::Value;
use crate::ast::ConnectType;

pub use expr::Expr;
pub use expr::IsExpr;

#[derive(Debug, Clone)]
pub struct Package {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    ModDef(Arc<ModDef>),
}

#[derive(Debug, Clone)]
pub struct ModDef {
    pub name: Ident,
    pub components: Vec<Component>,
    pub submodules: Vec<Submodule>,
    pub connects: Vec<Connect>,
}

#[derive(Debug, Clone)]
pub struct Submodule {
}

#[derive(Debug, Clone)]
pub enum Component {
    Incoming(Ident, Arc<Type>),
    Outgoing(Ident, Arc<Type>, Option<InlineConnect>),
    Wire(Ident, Arc<Type>, Option<InlineConnect>),
    Reg(Ident, Arc<Type>, Option<Expr>, /*Option<Value>,*/ Option<InlineConnect>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirType {
    Clock,
    Word(Width),
    Vec(Arc<Type>, usize),
}

#[derive(Debug, Clone)]
pub struct InlineConnect(pub ConnectType, pub Expr);

#[derive(Debug, Clone)]
pub struct Connect(pub Path, pub ConnectType, pub Expr);

impl Package {
    pub fn from_ast(package: &ast::Package) -> Arc<Package> {
        let mut items = vec![];
        for item in &package.items {
            items.push(Item::from_ast(item))
        }
        Package {
            items,
        }.into()
    }
}

impl Item {
    pub fn from_ast(item: &ast::Item) -> Item {
        match item {
            ast::Item::ModDef(moddef) => Item::ModDef(ModDef::from_ast(moddef)),
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

impl ModDef {
    pub fn from_ast(moddef: &ast::ModDef) -> Arc<ModDef> {
        let name = moddef.name.clone();
        let mut components = vec![];
        let mut submodules = vec![];
        let mut connects = vec![];

        for decl in &moddef.decls {
            match decl {
                ast::Decl::Component(component) => {
                    let c = match component.kind {
                        ast::ComponentKind::Incoming => Component::Incoming(component.name.clone(), Type::from_ast(&component.typ)),
                        ast::ComponentKind::Outgoing => Component::Outgoing(component.name.clone(), Type::from_ast(&component.typ), None),
                        ast::ComponentKind::Wire => todo!(),
                        ast::ComponentKind::Reg => Component::Reg(component.name.clone(), Type::from_ast(&component.typ), component.clock.as_ref().map(|e| Expr::from_ast(e)), None),
                    };
                    components.push(c);
                },
                ast::Decl::Submodule(submodule) => todo!(),
                ast::Decl::Connect(ast::Connect(path, connect_type, expr)) => {
                    connects.push(Connect(path.clone(), *connect_type, Expr::from_ast(expr)));
                },
            }
        }

        ModDef {
            name,
            components,
            submodules,
            connects,
        }.into()
    }
}

impl ast::Item {
    fn name(&self) -> Ident {
        match self {
            ast::Item::ModDef(m) => m.name.clone(),
        }
    }
}
