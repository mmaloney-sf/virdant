use crate::{ast, common::*, virdant_error};
use super::*;

#[salsa::query_group(ResolveQStorage)]
pub trait ResolveQ: item_resolution::ItemResolutionQ {
    fn resolve_package2(&self, package_name: Ident) -> VirdantResult<PackageId>;

    fn resolve_element(&self, moddef_id: ModDefId, name: Ident) -> VirdantResult<ElementId>;
    fn resolve_component(&self, moddef_id: ModDefId, target_id: Path) -> VirdantResult<ComponentId>;
}

fn resolve_package2(db: &dyn ResolveQ, package_name: Ident) -> VirdantResult<PackageId> {
    for package in db.packages() {
        if package.name() == package_name {
            return Ok(package);
        }
    }
    Err(virdant_error!("TODO resolve_package2"))
}

fn resolve_element(db: &dyn ResolveQ, moddef_id: ModDefId, name: Ident) -> VirdantResult<ElementId> {
    let moddef_ast = db.moddef_ast(moddef_id.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Submodule(submodule) if submodule.name == name => {
                return Ok(ElementId::from_ident(moddef_id.as_item(), name))
            },
            ast::Decl::Port(port) if port.name == name => {
                return Ok(ElementId::from_ident(moddef_id.as_item(), name))
            },
            _ => (),
        }
    }

    Err(virdant_error!("TODO resolve_element"))
}

fn resolve_component(db: &dyn ResolveQ, moddef_id: ModDefId, path: Path) -> VirdantResult<ComponentId> {
    eprintln!("resolve_component({moddef_id}, {path})");
    let parts = path.parts();
    if parts.len() == 1 {
        Ok(ComponentId::from_ident(moddef_id, parts[0].clone()))
    } else {
        let moddef_ast = db.moddef_ast(moddef_id.clone())?;
        for decl in &moddef_ast.decls {
            eprintln!("Trying to match {} and {:?}", parts[0], &decl);
            match decl {
                ast::Decl::Submodule(submodule) if submodule.name == parts[0] => {
                    eprintln!("Matched submodule name: {}", parts[0]);
                    let submodule_moddef_id: ModDefId = db.moddef(submodule.moddef.clone(), moddef_id.package())?;

                    return db.resolve_component(submodule_moddef_id.clone(), parts[1].clone().as_path());
                },
                _ => (),
            }
        }

        todo!()
    }
}
