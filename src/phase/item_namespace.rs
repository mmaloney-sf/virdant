use std::collections::HashSet;

use crate::ast;
use crate::common::*;
use super::*;

#[salsa::query_group(ItemNamespaceQStorage)]
pub trait ItemNamespaceQ: astq::AstQ {
    fn item_elements(&self, item_id: ItemId) -> VirdantResult<Vec<ComponentId>>;
}

fn item_elements(db: &dyn ItemNamespaceQ, item_id: ItemId) -> VirdantResult<Vec<ComponentId>> {
    match item_id {
        ItemId::ModDef(moddef) => moddef_components(db, moddef),
        ItemId::UnionDef(uniondef) => uniondef_elements(db, uniondef),
        ItemId::StructDef(structdef) => structdef_elements(db, structdef),
        ItemId::PortDef(portdef) => portdef_elements(db, portdef),
    }
}

fn moddef_components(db: &dyn ItemNamespaceQ, moddef_id: ModDefId) -> VirdantResult<Vec<ComponentId>> {
    let mut component_ids = HashSet::new();
    let mut errors = ErrorReport::new();

    let moddef_ast = db.moddef_ast(moddef_id.clone())?;
    for decl in &moddef_ast.decls {
         let name = match decl {
            ast::Decl::Component(component) => Some(&component.name),
            ast::Decl::Submodule(submodule) => Some(&submodule.name),
            ast::Decl::Port(port) => Some(&port.name),
            ast::Decl::Wire(_) => None,
        };

        if let Some(component_name) = name {
            let component_id = ComponentId::from_ident(moddef_id.clone(), component_name.clone());
            if !component_ids.insert(component_id) {
                errors.add(VirdantError::Other(format!("Duplicate component in {moddef_id}: {component_name}")));
            }
        }
    }

    errors.check()?;
    let component_ids: Vec<_> = component_ids.into_iter().collect();
    Ok(component_ids)
}

fn uniondef_elements(_db: &dyn ItemNamespaceQ, _uniondef_id: UnionDefId) -> VirdantResult<Vec<ComponentId>> {
    todo!()
}

fn structdef_elements(_db: &dyn ItemNamespaceQ, _structdef_id: StructDefId) -> VirdantResult<Vec<ComponentId>> {
    todo!()
}

fn portdef_elements(_db: &dyn ItemNamespaceQ, _portdef_id: PortDefId) -> VirdantResult<Vec<ComponentId>> {
    todo!()
}

