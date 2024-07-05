use crate::parse;
use crate::ast;
use crate::common::*;

use std::sync::Arc;

#[salsa::query_group(AstQStorage)]
pub trait AstQ: salsa::Database {
    #[salsa::input]
    fn source(&self) -> Arc<String>;

    fn package_ast(&self) -> VirdantResult<ast::Package>;
    fn moddef_ast(&self, moddef: Ident) -> VirdantResult<ast::ModDef>;
    fn moddef_component_ast(&self, moddef: Ident, component: Ident) -> VirdantResult<ast::SimpleComponent>;

    fn moddef_components(&self, moddef: Ident) -> VirdantResult<Vec<ast::SimpleComponent>>;
    fn moddef_submodules(&self, moddef: Ident) -> VirdantResult<Vec<ast::Submodule>>;
}

fn package_ast(db: &dyn AstQ) -> Result<ast::Package, VirdantError> {
    let input = db.source();
    parse::parse_package(&input)
}

fn moddef_ast(db: &dyn AstQ, moddef: Ident) -> Result<ast::ModDef, VirdantError> {
    let package = db.package_ast()?;
    let mut result: Option<ast::ModDef> = None;

    for item in &package.items {
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
            ast::Item::StructTypeDef(_structtypedef) => (),
        }
    }

    if let Some(moddef) = result {
        Ok(moddef)
    } else {
        Err(VirdantError::Other(format!("Unknown moddef {moddef}")))
    }
}

fn moddef_component_ast(db: &dyn AstQ, moddef: Ident, component: Ident) -> Result<ast::SimpleComponent, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(c) if c.name == component => return Ok(c.clone()),
            _ => (),
        }
    }
    Err(VirdantError::Other(format!("No such moddef {}", moddef)))
}


fn moddef_components(db: &dyn AstQ, moddef: Ident) -> VirdantResult<Vec<ast::SimpleComponent>> {
    let moddef_ast = db.moddef_ast(moddef)?;
    let mut results = vec![];
    for decl in &moddef_ast.decls {
        if let ast::Decl::SimpleComponent(component) = decl {
            results.push(component.clone());
        }
    }
    Ok(results)
}

fn moddef_submodules(db: &dyn AstQ, moddef: Ident) -> VirdantResult<Vec<ast::Submodule>> {
    let moddef_ast = db.moddef_ast(moddef)?;
    let mut results = vec![];
    for decl in &moddef_ast.decls {
        if let ast::Decl::Submodule(submodule) = decl {
            results.push(submodule.clone());
        }
    }
    Ok(results)
}
