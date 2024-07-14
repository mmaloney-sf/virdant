use std::collections::HashSet;
use crate::common::*;
use crate::ast;
use super::*;
use std::sync::Arc;

#[salsa::query_group(ItemDependencyQStorage)]
pub trait ItemDependencyQ: super::item_resolution::ItemResolutionQ {
    fn moddef_item_dependencies(&self, moddef: ModDef) -> VirdantResult<Vec<Item>>;
}

fn moddef_item_dependencies(db: &dyn ItemDependencyQ, moddef: ModDef) -> VirdantResult<Vec<Item>> {
    let mut errors = ErrorReport::new();
    let mut dependencies: HashSet<Item> = HashSet::new();
    let moddef_ast = db.moddef_ast(moddef.package(), moddef.name())?;

    let package_path: Path = moddef.fqname().parent();

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(simplecomponent) => {
                match moddef_item_dependencies_simplecomponent(db, moddef.clone(), simplecomponent) {
                    Ok(deps) => dependencies.extend(deps),
                    Err(e) => errors.add(e),
                }
            },
            ast::Decl::Submodule(submodule) => {
                let item = db.item(package_path.join(&submodule.moddef.as_path()))?;
                dependencies.insert(item);
            },
            ast::Decl::Wire(ast::Wire(_target, _wire_type, expr)) => {
                match expr_item_dependencies(db, expr.clone()) {
                    Ok(deps) => dependencies.extend(deps),
                    Err(e) => errors.add(e),
                }
            },
        }
    }

    errors.check()?;
    Ok(dependencies.into_iter().collect())
}

fn moddef_item_dependencies_simplecomponent(db: &dyn ItemDependencyQ, moddef: ModDef, simplecomponent: &ast::SimpleComponent) -> VirdantResult<Vec<Item>> {
    eprintln!("moddef_item_dependencies_simplecomponent({})", simplecomponent.name);
    let package_path: Path = moddef.fqname().parent();
    let mut items = vec![];
    eprintln!("    typ is {:?}", simplecomponent.typ);
    if let ast::Type::TypeRef(name) = simplecomponent.typ.as_ref() {
        let item = db.item(package_path.join(&name.as_path()))?;
        items.push(item);
    }

    if let Some(clock) = simplecomponent.clock.clone() {
        let expr_depends = expr_item_dependencies(db, ast::Expr::Reference(clock).into())?;
        items.extend(expr_depends);
    }

    eprintln!("    = {items:?}");
    Ok(items)
}

fn expr_item_dependencies(_db: &dyn ItemDependencyQ, _expr: Arc<ast::Expr>) -> VirdantResult<Vec<Item>> {
    eprintln!("TODO expr_item_dependencies");
    Ok(vec![])
}