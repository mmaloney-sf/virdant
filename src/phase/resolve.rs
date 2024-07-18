use crate::{ast, common::*};
use super::*;

#[salsa::query_group(ResolveQStorage)]
pub trait ResolveQ: item_resolution::ItemResolutionQ {
    fn resolve_package2(&self, package_name: Ident) -> VirdantResult<PackageId>;

    fn resolve_element(&self, moddef_id: ModDefId, name: Ident) -> VirdantResult<ComponentId>;
    fn resolve_component(&self, moddef_id: ModDefId, target_id: PathId) -> VirdantResult<ComponentId>;
    fn resolve_path(&self, moddef_id: ModDefId, path: Path) -> VirdantResult<PathId>;
}

fn resolve_package2(db: &dyn ResolveQ, package_name: Ident) -> VirdantResult<PackageId> {
    for package in db.packages() {
        if package.name() == package_name {
            return Ok(package);
        }
    }
    Err(VirdantError::Unknown)
}

fn resolve_element(db: &dyn ResolveQ, moddef_id: ModDefId, name: Ident) -> VirdantResult<ComponentId> {
    let moddef_ast = db.moddef_ast(moddef_id.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Submodule(submodule) if submodule.name == name => {
                return Ok(ComponentId::from_ident(moddef_id, name))
            },
            ast::Decl::Port(port) if port.name == name => {
                return Ok(ComponentId::from_ident(moddef_id, name))
            },
            _ => (),
        }
    }

    Err(VirdantError::Unknown)
}

fn resolve_component(db: &dyn ResolveQ, moddef_id: ModDefId, target_id: PathId) -> VirdantResult<ComponentId> {
    eprintln!("resolve_component({moddef_id}, {target_id})");
    let path: Path = target_id.as_path();
    let parts = path.parts();
    if parts.len() == 1 {
        Ok(ComponentId::from_ident(moddef_id, parts[0].clone()))
    } else {
        eprintln!("    it's a dot into a submodule");
        let moddef_ast = db.moddef_ast(moddef_id.clone())?;
        for decl in &moddef_ast.decls {
            match decl {
                ast::Decl::Submodule(submodule) if submodule.name == parts[0] => {
                    eprintln!("    the submodule's module def name is {}", &submodule.moddef);
                    let submodule_moddef_id: ModDefId = db.moddef(submodule.moddef.clone(), moddef_id.package())?;
                    eprintln!("    the submodule's moddef_id is {submodule_moddef_id}");
                    eprintln!("    RECURSING");

                    return db.resolve_component(submodule_moddef_id.clone(), PathId::from_path(submodule_moddef_id, parts[1].clone().as_path()));
                },
                _ => (),
            }
        }

        todo!()
    }
}

fn resolve_path(_db: &dyn ResolveQ, moddef_id: ModDefId, path: Path) -> VirdantResult<PathId> {
    Ok(PathId::from_path(moddef_id, path))
}
