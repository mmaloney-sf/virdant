use std::collections::HashSet;

use crate::ast;
use crate::common::*;
use super::*;

#[salsa::query_group(ItemNamespaceQStorage)]
pub trait ItemNamespaceQ: astq::AstQ {
    fn item_names_unique(&self, item_id: ItemId) -> VirdantResult<()>;
}

fn item_names_unique(db: &dyn ItemNamespaceQ, item_id: ItemId) -> VirdantResult<()> {
    match item_id {
        ItemId::ModDef(moddef) => moddef_names_unique(db, moddef),
        ItemId::UnionDef(uniondef) => uniondef_names_unique(db, uniondef),
        ItemId::StructDef(structdef) => structdef_names_unique(db, structdef),
        ItemId::PortDef(portdef) => portdef_names_unique(db, portdef),
    }
}

fn moddef_names_unique(db: &dyn ItemNamespaceQ, moddef_id: ModDefId) -> VirdantResult<()> {
    let mut names = HashSet::new();
    let mut errors = ErrorReport::new();

    let moddef_ast = db.moddef_ast(moddef_id.clone())?;
    for decl in &moddef_ast.decls {
         let name = match decl {
            ast::Decl::SimpleComponent(simplecomponent) => Some(&simplecomponent.name),
            ast::Decl::Submodule(submodule) => Some(&submodule.name),
            ast::Decl::Port(port) => Some(&port.name),
            ast::Decl::Wire(_) => None,
        };

        if let Some(component_name) = name {
            if !names.insert(component_name.clone()) {
                errors.add(VirdantError::Other(format!("Duplicate component in {moddef_id}: {component_name}")));
            }
        }
    }

    errors.check()?;
    Ok(())
}

fn uniondef_names_unique(db: &dyn ItemNamespaceQ, uniondef_id: UnionDefId) -> VirdantResult<()> {
    todo!()
}

fn structdef_names_unique(db: &dyn ItemNamespaceQ, structdef_id: StructDefId) -> VirdantResult<()> {
    todo!()
}

fn portdef_names_unique(db: &dyn ItemNamespaceQ, portdef_id: PortDefId) -> VirdantResult<()> {
    todo!()
}

