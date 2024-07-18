use crate::ast::ComponentKind;
use crate::parse;
use crate::ast;
use crate::common::*;
use crate::virdant_error;
use super::*;

use std::collections::HashMap;
use std::sync::Arc;

#[salsa::query_group(AstQStorage)]
pub trait AstQ: salsa::Database {
    #[salsa::input]
    fn sources(&self) -> HashMap<String, Arc<String>>;

    fn packages(&self) -> Vec<PackageId>;

    fn package_ast(&self, package_id: PackageId) -> VirdantResult<ast::Package>;
    fn moddef_ast(&self, moddef_id: ModDefId) -> VirdantResult<ast::ModDef>;
    fn uniondef_ast(&self, uniondef_id: UnionDefId) -> VirdantResult<ast::UnionDef>;

    fn component_ast(&self, component_id: ComponentId) -> VirdantResult<ast::Component>;

    fn wire_ast(&self, moddef_id: ModDefId, path_id: Path) -> VirdantResult<Option<ast::Wire>>;
}

fn packages(db: &dyn AstQ) -> Vec<PackageId> {
    let mut packages: Vec<String> = vec![];

    for package_name in db.sources().keys() {
        packages.push(package_name.clone());
    }

    packages.sort();
    packages.into_iter().map(|package| PackageId::from_ident(package.into())).collect()
}

fn package_ast(db: &dyn AstQ, package_id: PackageId) -> Result<ast::Package, VirdantError> {
    eprintln!("package_ast({package_id})");
    let sources = db.sources();
    let package_name = package_id.name().to_string();
    if let Some(input) = sources.get(&package_name) {
        parse::parse_package(&input)
    } else {
        Err(virdant_error!("TODO package_ast"))
    }
}

fn moddef_ast(db: &dyn AstQ, moddef_id: ModDefId) -> Result<ast::ModDef, VirdantError> {
    let package_ast = db.package_ast(moddef_id.package())?;
    let mut result: Option<ast::ModDef> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                if moddef_ast.name == moddef_id.name() {
                    if result.is_none() {
                        result = Some(moddef_ast.clone());
                    } else {
                        return Err(VirdantError::Other("Uh oh".into()));
                    }
                }
            },
            _ => (),
        }
    }

    if let Some(moddef) = result {
        Ok(moddef)
    } else {
        Err(VirdantError::Other(format!("Unknown moddef {moddef_id}")))
    }
}

fn uniondef_ast(db: &dyn AstQ, uniontype_id: UnionDefId) -> Result<ast::UnionDef, VirdantError> {
    let package_ast = db.package_ast(uniontype_id.package())?;
    let mut result: Option<ast::UnionDef> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::UnionDef(alttypedef_ast) => {
                if alttypedef_ast.name == uniontype_id.name() {
                    if result.is_none() {
                        result = Some(alttypedef_ast.clone());
                    } else {
                        return Err(VirdantError::Other("Uh oh".into()));
                    }
                }
            },
            _ => (),
        }
    }

    if let Some(alttypedef) = result {
        Ok(alttypedef)
    } else {
        Err(VirdantError::Other(format!("Unknown alt type {uniontype_id}")))
    }
}

fn component_ast(db: &dyn AstQ, component_id: ComponentId) -> VirdantResult<ast::Component> {
    let moddef_ast = db.moddef_ast(component_id.moddef())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(component) if component.name == component_id.name() => {
                return Ok(component.clone());
            },
            _ => (),
        }
    }
    Err(virdant_error!("No component: {component_id}"))
}

fn wire_ast(db: &dyn AstQ, moddef_id: ModDefId, path: Path) -> VirdantResult<Option<ast::Wire>> {
    let moddef_ast = db.moddef_ast(moddef_id)?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Wire(wire@ast::Wire(target, _, _)) if target == &path => {
                return Ok(Some(wire.clone()));
            },
            ast::Decl::Component(component) => {
                // if we detect this is an `incoming` (which has no driver), return None.
                if component.kind == ComponentKind::Incoming && component.name.as_path() == path {
                    return Ok(None);
                }
            },
            _ => (),
        }
    }
    Err(virdant_error!("No such wire: {}", path))
}
