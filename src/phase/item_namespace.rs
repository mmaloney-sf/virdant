use std::collections::HashSet;

use crate::ast;
use crate::common::*;
use super::*;

#[salsa::query_group(ItemNamespaceQStorage)]
pub trait ItemNamespaceQ: astq::AstQ {
    fn item_elements(&self, item_id: ItemId) -> VirdantResult<Vec<ElementId>>;
}

fn item_elements(db: &dyn ItemNamespaceQ, item_id: ItemId) -> VirdantResult<Vec<ElementId>> {
    match item_id {
        ItemId::ModDef(moddef) => {
            let component_ids = moddef_components(db, moddef)?;
            let element_ids = component_ids.into_iter().map(|component_id| component_id.as_element()).collect();
            Ok(element_ids)
        },
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

fn uniondef_elements(db: &dyn ItemNamespaceQ, uniondef_id: UnionDefId) -> VirdantResult<Vec<ElementId>> {
    let mut elements = vec![];
    let uniondef_ast = db.uniondef_ast(uniondef_id.clone())?;
    for ast::Alt(name, _sig) in uniondef_ast.alts {
        elements.push(ElementId::from_ident(uniondef_id.clone().as_item(), name));
    }
    Ok(elements)
}

fn structdef_elements(db: &dyn ItemNamespaceQ, structdef_id: StructDefId) -> VirdantResult<Vec<ElementId>> {
    let mut elements = vec![];
    let structdef_ast = db.structdef_ast(structdef_id.clone())?;
    for ast::Field(name, _typ) in structdef_ast.fields {
        elements.push(ElementId::from_ident(structdef_id.clone().as_item(), name));
    }
    Ok(elements)
}

fn portdef_elements(_db: &dyn ItemNamespaceQ, _portdef_id: PortDefId) -> VirdantResult<Vec<ElementId>> {
    todo!()
}
