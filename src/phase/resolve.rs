use crate::{ast, common::*};
use super::*;

#[salsa::query_group(ResolveQStorage)]
pub trait ResolveQ: astq::AstQ {
    fn resolve_package2(&self, package_name: Ident) -> VirdantResult<PackageId>;

    fn resolve_element(&self, moddef_id: ModDefId, name: Ident) -> VirdantResult<ElementId>;
    fn resolve_target(&self, moddef_id: ModDefId, path: Path) -> VirdantResult<ModDefElementId>;
}

fn resolve_package2(db: &dyn ResolveQ, package_name: Ident) -> VirdantResult<PackageId> {
    for package in db.packages() {
        if package.name() == package_name {
            return Ok(package);
        }
    }
    Err(VirdantError::Unknown)
}

fn resolve_element(db: &dyn ResolveQ, moddef_id: ModDefId, name: Ident) -> VirdantResult<ElementId> {
    let moddef_ast = db.moddef_ast(moddef_id.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Submodule(submodule) if submodule.name == name => {
                return Ok(ElementId::ModDef(ModDefElementId::from_ident(moddef_id, name)))
            },
            ast::Decl::Port(port) if port.name == name => {
                return Ok(ElementId::ModDef(ModDefElementId::from_ident(moddef_id, name)))
            },
            _ => (),
        }
    }

    Err(VirdantError::Unknown)
}

fn resolve_target(_db: &dyn ResolveQ, moddef_id: ModDefId, path: Path) -> VirdantResult<ModDefElementId> {
    let parts = path.parts();
    if parts.len() == 1 {
        Ok(ModDefElementId::from_ident(moddef_id, path.head()))
    } else {
        todo!()
    }
}
