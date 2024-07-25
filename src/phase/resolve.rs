use crate::{ast, common::*, virdant_error};
use super::*;

#[salsa::query_group(ResolveQStorage)]
pub trait ResolveQ: item_resolution::ItemResolutionQ {
    fn resolve_package2(&self, package_name: Ident) -> VirdantResult<PackageId>;

    fn resolve_element(&self, item_id: ItemId, name: Ident) -> VirdantResult<ElementId>;
    fn element_ofness(&self, element_id: ElementId) -> VirdantResult<Option<ItemId>>;

    fn resolve_component_by_path(&self, moddef_id: ModDefId, path: Path) -> VirdantResult<ElementId>;
}

fn resolve_component_by_path(db: &dyn ResolveQ, moddef_id: ModDefId, path: Path) -> VirdantResult<ElementId> {
    eprintln!("resolve_component_by_path({moddef_id}, {path})");
    let mut item_id: ItemId = moddef_id.as_item();
    let mut element_id: Option<ElementId> = None;

    for part in path.parts() {
        let el = db.resolve_element(item_id.clone(), part)?;
        if let Some(new_item_id) = db.element_ofness(el.clone())? {
            item_id = new_item_id;
        }
        element_id = Some(el);
    }

    eprintln!("resolve_component_by_path({moddef_id}, {path}) = {element_id:?}");
    Ok(element_id.unwrap())
}

fn element_ofness(db: &dyn ResolveQ, element_id: ElementId) -> VirdantResult<Option<ItemId>> {
    let item_ast = db.item_ast(element_id.item())?;
    let package_id = element_id.item().package();

    if let ast::Item::ModDef(moddef_ast) = item_ast {
        for decl in &moddef_ast.decls {
            match decl {
                ast::Decl::Submodule(submodule) if submodule.name == element_id.name() => {
                    return Ok(Some(db.item(submodule.moddef.clone(), package_id)?))
                },
                ast::Decl::Port(port) if port.name == element_id.name() => {
                    return Ok(Some(db.item(port.portdef.clone(), package_id)?))
                },
                _ => (),
            }
        }
    }

    Ok(None)
}

fn resolve_package2(db: &dyn ResolveQ, package_name: Ident) -> VirdantResult<PackageId> {
    for package in db.packages() {
        if package.name() == package_name {
            return Ok(package);
        }
    }
    Err(virdant_error!("TODO resolve_package2"))
}

fn resolve_element(db: &dyn ResolveQ, item_id: ItemId, name: Ident) -> VirdantResult<ElementId> {
    let item_ast = db.item_ast(item_id.clone())?;

    if let ast::Item::ModDef(moddef_ast) = item_ast {
        for decl in &moddef_ast.decls {
            match decl {
                ast::Decl::Component(component) if component.name == name => {
                    return Ok(ElementId::from_ident(item_id, name))
                },
                ast::Decl::Submodule(submodule) if submodule.name == name => {
                    return Ok(ElementId::from_ident(item_id, name))
                },
                ast::Decl::Port(port) if port.name == name => {
                    return Ok(ElementId::from_ident(item_id, name))
                },
                _ => (),
            }
        }
    } else if let ast::Item::PortDef(portdef_ast) = item_ast {
        for channel in &portdef_ast.channels {
            let ast::Channel(_dir, channel_name, _typ) = channel;
            if channel_name == &name {
                return Ok(ElementId::from_ident(item_id, name))
            }
        }
    }

    Err(virdant_error!("TODO resolve_element: Could not resolve {name} in {item_id}"))
}
