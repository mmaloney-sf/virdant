use crate::parse;
use crate::ast;
use crate::common::*;
use super::*;

use std::sync::Arc;

#[salsa::query_group(AstQStorage)]
pub trait AstQ: salsa::Database {
    #[salsa::input]
    fn package_source(&self, package: Path) -> Arc<String>;

    fn package_ast(&self, package: Package) -> VirdantResult<ast::Package>;
    fn moddef_ast(&self, package: Package, moddef: Ident) -> VirdantResult<ast::ModDef>;
    fn uniondef_ast(&self, package: Package, moddef: Ident) -> VirdantResult<ast::AltTypeDef>;
}

fn package_ast(db: &dyn AstQ, package: Package) -> Result<ast::Package, VirdantError> {
    let input = db.package_source(package.into());
    parse::parse_package(&input)
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
