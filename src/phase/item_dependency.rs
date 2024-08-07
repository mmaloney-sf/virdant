use std::collections::HashSet;
use crate::common::*;
use crate::ast::Ast;
use crate::ast;
use super::*;

#[salsa::query_group(ItemDependencyQStorage)]
pub trait ItemDependencyQ: resolve::ResolveQ {
    fn item_dependencies(&self, item: ItemId) -> VirdantResult<Vec<ItemId>>;
}

fn item_dependencies(db: &dyn ItemDependencyQ, item: ItemId) -> VirdantResult<Vec<ItemId>> {
    match item {
        ItemId::ModDef(moddef_id) => moddef_item_dependencies(db, moddef_id),
        ItemId::UnionDef(uniondef_id) => uniondef_item_dependencies(db, uniondef_id),
        ItemId::StructDef(structdef_id) => structdef_item_dependencies(db, structdef_id),
        ItemId::PortDef(portdef_id) => portdef_item_dependencies(db, portdef_id),
    }
}

fn moddef_item_dependencies(db: &dyn ItemDependencyQ, moddef: ModDefId) -> VirdantResult<Vec<ItemId>> {
    let mut errors = ErrorReport::new();
    let mut dependencies: HashSet<ItemId> = HashSet::new();
    let moddef_ast = db.moddef_ast(moddef.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(component) => {
                match moddef_item_dependencies_component(db, moddef.clone(), component) {
                    Ok(deps) => dependencies.extend(deps),
                    Err(e) => errors.add(e),
                }
            },
            ast::Decl::Submodule(submodule) => {
                let item = db.item(submodule.moddef.clone(), moddef.package())?;
                dependencies.insert(item);
            },
            ast::Decl::Wire(wire) => {
                let ast::Wire(_target, _wire_type, expr) = wire.as_ref();
                match expr_item_dependencies(db, expr.clone()) {
                    Ok(deps) => dependencies.extend(deps),
                    Err(e) => errors.add(e),
                }
            },
            ast::Decl::Port(port) => {
                let item = db.item(port.portdef.clone(), moddef.package())?;
                dependencies.insert(item);
            },
        }
    }

    errors.check()?;
    Ok(dependencies.into_iter().collect())
}

fn moddef_item_dependencies_component(
    db: &dyn ItemDependencyQ,
    moddef: ModDefId,
    simplecomponent: &ast::Component,
) -> VirdantResult<Vec<ItemId>> {
    let mut items = vec![];
    if let ast::Type::TypeRef(name) = simplecomponent.typ.as_ref() {
        let item = db.item(name.clone(), moddef.package())?;
        items.push(item);
    }

    if let Some(clock) = simplecomponent.clock.clone() {
        let expr_depends = expr_item_dependencies(db, clock.into())?;
        items.extend(expr_depends);
    }

    Ok(items)
}

fn expr_item_dependencies(_db: &dyn ItemDependencyQ, _expr: Ast<ast::Expr>) -> VirdantResult<Vec<ItemId>> {
    Ok(vec![])
}

fn uniondef_item_dependencies(_db: &dyn ItemDependencyQ, _uniondef_id: UnionDefId) -> VirdantResult<Vec<ItemId>> {
    eprintln!("TODO uniondef_item_dependencies not implemented");
    Ok(vec![])
}

fn structdef_item_dependencies(_db: &dyn ItemDependencyQ, _structdef_id: StructDefId) -> VirdantResult<Vec<ItemId>> {
    eprintln!("TODO structdef_item_dependencies not implemented");
    Ok(vec![])
}

fn portdef_item_dependencies(_db: &dyn ItemDependencyQ, _portdef_id: PortDefId) -> VirdantResult<Vec<ItemId>> {
    eprintln!("TODO portdef_item_dependencies not implemented");
    Ok(vec![])
}
