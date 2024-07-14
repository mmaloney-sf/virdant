use crate::parse;
use crate::ast;
use crate::common::*;
use super::*;

use std::collections::HashMap;
use std::sync::Arc;

#[salsa::query_group(AstQStorage)]
pub trait AstQ: salsa::Database {
    #[salsa::input]
    fn sources(&self) -> HashMap<String, Arc<String>>;

    fn packages(&self) -> Vec<Package>;

    fn package_ast(&self, package: Package) -> VirdantResult<ast::Package>;
    fn moddef_ast(&self, package: Package, moddef: Ident) -> VirdantResult<ast::ModDef>;
    fn uniondef_ast(&self, package: Package, moddef: Ident) -> VirdantResult<ast::AltTypeDef>;
}

fn packages(db: &dyn AstQ) -> Vec<Package> {
    let mut packages: Vec<String> = vec![];

    for package_name in db.sources().keys() {
        packages.push(package_name.clone());
    }

    packages.sort();
    packages.into_iter().map(|package| Path::from(package).into()).collect()
}

fn package_ast(db: &dyn AstQ, package: Package) -> Result<ast::Package, VirdantError> {
    let sources = db.sources();
    let package_name: String = Path::from(package).to_string();
    if let Some(input) = sources.get(&package_name) {
        parse::parse_package(&input)
    } else {
        Err(VirdantError::Unknown)
    }
}

fn moddef_ast(db: &dyn AstQ, package: Package, moddef: Ident) -> Result<ast::ModDef, VirdantError> {
    let package_ast = db.package_ast(package)?;
    let mut result: Option<ast::ModDef> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                if moddef_ast.name == moddef {
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

fn uniondef_ast(db: &dyn AstQ, package: Package, uniontype: Ident) -> Result<ast::AltTypeDef, VirdantError> {
    let package_ast = db.package_ast(package)?;
    let mut result: Option<ast::AltTypeDef> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::AltTypeDef(alttypedef_ast) => {
                if alttypedef_ast.name == uniontype {
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
