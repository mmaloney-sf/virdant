use crate::{ast, common::*};
use super::*;

#[salsa::query_group(ResolveQStorage)]
pub trait ResolveQ: astq::AstQ {
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

fn resolve_component(_db: &dyn ResolveQ, moddef_id: ModDefId, target_id: PathId) -> VirdantResult<ComponentId> {
    let path: Path = target_id.as_path();
    let parts = path.parts();
    if parts.len() == 1 {
        Ok(ComponentId::from_ident(moddef_id, parts[0].clone()))
    } else {
        todo!()
    }
}

fn resolve_path(_db: &dyn ResolveQ, moddef_id: ModDefId, path: Path) -> VirdantResult<PathId> {
    Ok(PathId::from_path(moddef_id, path))
}
