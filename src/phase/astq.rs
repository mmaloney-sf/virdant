use crate::parse;
use crate::ast;
use crate::common::*;
use super::*;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

#[salsa::query_group(AstQStorage)]
pub trait AstQ: salsa::Database {
    #[salsa::input]
    fn sources(&self) -> HashMap<String, Arc<String>>;

    fn packages(&self) -> Vec<PackageId>;

    fn package_ast(&self, package: PackageId) -> VirdantResult<ast::Package>;
    fn moddef_ast(&self, moddef: ModDefId) -> VirdantResult<ast::ModDef>;
    fn uniondef_ast(&self, uniondef: UnionDefId) -> VirdantResult<ast::UnionDef>;

    // TODO MOVE THIS
    fn imports(&self, package: PackageId) -> VirdantResult<Vec<PackageId>>;
}

fn packages(db: &dyn AstQ) -> Vec<PackageId> {
    let mut packages: Vec<String> = vec![];

    for package_name in db.sources().keys() {
        packages.push(package_name.clone());
    }

    packages.sort();
    packages.into_iter().map(|package| Path::from(package).into()).collect()
}

fn package_ast(db: &dyn AstQ, package: PackageId) -> Result<ast::Package, VirdantError> {
    let sources = db.sources();
    let package_name: String = Path::from(package).to_string();
    if let Some(input) = sources.get(&package_name) {
        parse::parse_package(&input)
    } else {
        Err(VirdantError::Unknown)
    }
}

fn moddef_ast(db: &dyn AstQ, moddef: ModDefId) -> Result<ast::ModDef, VirdantError> {
    let package_ast = db.package_ast(moddef.package())?;
    let mut result: Option<ast::ModDef> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                if moddef_ast.name == moddef.name() {
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
        Err(VirdantError::Other(format!("Unknown moddef {moddef}")))
    }
}

fn uniondef_ast(db: &dyn AstQ, uniontype: UnionDefId) -> Result<ast::UnionDef, VirdantError> {
    let package_ast = db.package_ast(uniontype.package())?;
    let mut result: Option<ast::UnionDef> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::UnionDef(alttypedef_ast) => {
                if alttypedef_ast.name == uniontype.name() {
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
        Err(VirdantError::Other(format!("Unknown alt type {uniontype}")))
    }
}


fn imports(db: &dyn AstQ, package: PackageId) -> VirdantResult<Vec<PackageId>> {
    let package_ast = db.package_ast(package)?;
    let mut errors = ErrorReport::new();
    let mut packages = HashSet::new();
    for ast::PackageImport(package_name) in &package_ast.imports {
        if !packages.insert(package_name.as_path().into()) {
            errors.add(VirdantError::Other(format!("Duplicate import: {package_name}")));
        }
    }
    errors.check()?;
    Ok(packages.into_iter().collect())
}
