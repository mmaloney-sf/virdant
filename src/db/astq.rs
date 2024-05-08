use crate::parse;
use crate::ast;
use crate::common::*;

use std::sync::Arc;

#[salsa::query_group(AstQStorage)]
pub trait AstQ: salsa::Database {
    #[salsa::input]
    fn source(&self) -> Arc<String>;
    fn package_ast(&self) -> Result<ast::Package, VirdantError>;
    fn moddef_ast(&self, moddef: Ident) -> Result<ast::ModDef, VirdantError>;
}

fn package_ast(db: &dyn AstQ) -> Result<ast::Package, VirdantError> {
    let input = db.source();
    parse::parse_package(&input)
}

fn moddef_ast(db: &dyn AstQ, key: Ident) -> Result<ast::ModDef, VirdantError> {
    let package = db.package_ast()?;
    let mut result: Option<ast::ModDef> = None;

    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => {
                if moddef.name == key {
                    if result.is_none() {
                        result = Some(moddef.clone());
                    } else {
                        return Err(VirdantError::Other("Uh oh".into()));
                    }
                }
            }
        }
    }

    if let Some(moddef) = result {
        Ok(moddef)
    } else {
        Err(VirdantError::Unknown)
    }
}
