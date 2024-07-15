use std::collections::HashSet;
use crate::common::*;
use crate::ast;
use super::*;
use std::sync::Arc;

#[salsa::query_group(ItemDependencyQStorage)]
pub trait ItemDependencyQ: super::item_resolution::ItemResolutionQ {
    fn moddef_item_dependencies(&self, moddef: ModDefId) -> VirdantResult<Vec<ItemId>>;
}

fn moddef_item_dependencies(db: &dyn ItemDependencyQ, moddef: ModDefId) -> VirdantResult<Vec<ItemId>> {
    let mut errors = ErrorReport::new();
    let mut dependencies: HashSet<ItemId> = HashSet::new();
    let moddef_ast = db.moddef_ast(moddef.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(simplecomponent) => {
                match moddef_item_dependencies_simplecomponent(db, moddef.clone(), simplecomponent) {
                    Ok(deps) => dependencies.extend(deps),
                    Err(e) => errors.add(e),
                }
            },
            ast::Decl::Submodule(submodule) => {
                let item = db.item(submodule.moddef.clone(), moddef.package())?;
                dependencies.insert(item);
            },
            ast::Decl::Wire(ast::Wire(_target, _wire_type, expr)) => {
                match expr_item_dependencies(db, expr.clone()) {
                    Ok(deps) => dependencies.extend(deps),
                    Err(e) => errors.add(e),
                }
            },
            ast::Decl::Port(_) => todo!(),
        }
    }

    errors.check()?;
    Ok(dependencies.into_iter().collect())
}

fn moddef_item_dependencies_simplecomponent(
    db: &dyn ItemDependencyQ,
    moddef: ModDefId,
    simplecomponent: &ast::SimpleComponent,
) -> VirdantResult<Vec<ItemId>> {
    let mut items = vec![];
    if let ast::Type::TypeRef(name) = simplecomponent.typ.as_ref() {
        let item = db.item(name.clone(), moddef.package())?;
        items.push(item);
    }

    if let Some(clock) = simplecomponent.clock.clone() {
        let expr_depends = expr_item_dependencies(db, ast::Expr::Reference(clock).into())?;
        items.extend(expr_depends);
    }

    Ok(items)
}

fn expr_item_dependencies(_db: &dyn ItemDependencyQ, _expr: Arc<ast::Expr>) -> VirdantResult<Vec<ItemId>> {
    Ok(vec![])
}
