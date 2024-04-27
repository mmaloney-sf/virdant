mod expr;

use std::collections::HashMap;
use std::sync::Arc;
use crate::common::*;
use crate::ast;
use crate::types::Type;
use crate::value::Value;
use crate::ast::InlineConnect;

pub use expr::Expr;
pub use expr::IsExpr;

pub struct Package {
    items: HashMap<Ident, Arc<Item>>,
}

#[derive(Debug, Clone)]
pub enum Item {
}

#[derive(Debug, Clone)]
pub struct ModDef {
    pub name: Ident,
    pub components: Vec<Component>,
    pub submodules: Vec<Submodule>,
}

#[derive(Debug, Clone)]
pub struct Submodule {
}

#[derive(Debug, Clone)]
pub enum Component {
    Incoming(Ident, Type),
    Outgoing(Ident, Type, InlineConnect),
    Wire(Ident, Type, InlineConnect),
    Reg(Ident, Type, Expr, Value, InlineConnect),
}

impl Package {
    pub fn to_hir(package: &ast::Package) -> Result<Arc<Package>, VirdantError> {
        let mut errors = ErrorReport::new();
        let mut items = HashMap::new();
        for item in &package.items {
            let previous = items.insert(item.name(), Item::to_hir(item)?);
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
    pub fn to_hir(item: &ast::Item) -> Result<Arc<Item>, VirdantError> {
        todo!()
    }
}

impl ast::Item {
    fn name(&self) -> Ident {
        match self {
            ast::Item::ModDef(m) => m.name.clone(),
        }
    }
}
