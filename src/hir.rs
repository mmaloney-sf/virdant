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
    pub items: HashMap<Ident, Item>,
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
    Reg(Ident, Arc<Type>, Expr, /*Option<Value>,*/ Option<InlineConnect>),
}

#[derive(Debug, Clone)]
pub struct InlineConnect(pub ConnectType, pub Expr);

#[derive(Debug, Clone)]
pub struct Connect(pub Path, pub ConnectType, pub Expr);

impl Package {
    pub fn from_ast(package: &ast::Package) -> Result<Arc<Package>, VirdantError> {
        let mut errors = ErrorReport::new();
        let mut items = HashMap::new();
        for item in &package.items {
            let previous = items.insert(item.name(), Item::from_ast(item)?);
            if let Some(previous_item) = previous {
                errors.add(VirdantError::Unknown(format!("Item already defined: {previous_item:?}")));
            }
        }
        errors.check()?;
        let package = Package {
            items,
        };
        Ok(package.into())
    }
}

impl Item {
    pub fn from_ast(item: &ast::Item) -> Result<Item, VirdantError> {
        match item {
            ast::Item::ModDef(moddef) => Ok(Item::ModDef(ModDef::from_ast(moddef)?)),
        }
    }
}

impl Type {
    pub fn from_ast(typ: &ast::Type) -> Result<Arc<Type>, VirdantError> {
        Ok(match typ {
            ast::Type::Clock => Type::Clock.into(),
            ast::Type::Word(width) => Type::Word(*width).into(),
            ast::Type::Vec(inner, len) => Type::Vec(Type::from_ast(inner)?, *len).into(),
        })
    }
}

impl ModDef {
    pub fn from_ast(moddef: &ast::ModDef) -> Result<Arc<ModDef>, VirdantError> {
        let mut errors = ErrorReport::new();

        let name = moddef.name.clone();
        let mut components = vec![];
        let mut submodules = vec![];
        let mut connects = vec![];

        for decl in &moddef.decls {
            match decl {
                ast::Decl::Component(component) => {
                    match component.kind {
                        ast::ComponentKind::Incoming => {
                            let c = Component::Incoming(component.name.clone(), Type::from_ast(&component.typ)?);
                            components.push(c);
                        },
                        ast::ComponentKind::Outgoing => {
                            let c = Component::Outgoing(component.name.clone(), Type::from_ast(&component.typ)?, None);
                            components.push(c);
                        },
                        ast::ComponentKind::Wire => todo!(),
                        ast::ComponentKind::Reg => {
                            if let Some(clock) = &component.clock {
                                let c = Component::Reg(component.name.clone(), Type::from_ast(&component.typ)?, Expr::from_ast(clock)?, None);
                                components.push(c);
                            } else {
                                errors.add(VirdantError::Unknown(format!("Reg {} has no clock", component.name)));
                            }
                        },
                    }
                },
                ast::Decl::Submodule(submodule) => todo!(),
                ast::Decl::Connect(ast::Connect(path, connect_type, expr)) => {
                    connects.push(Connect(path.clone(), *connect_type, Expr::from_ast(expr)?));
                },
            }
        }

        errors.check()?;

        Ok(ModDef {
            name,
            components,
            submodules,
            connects,
        }.into())
    }
}

impl ast::Item {
    fn name(&self) -> Ident {
        match self {
            ast::Item::ModDef(m) => m.name.clone(),
        }
    }
}
