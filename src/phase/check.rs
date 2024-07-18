//use crate::topological_sort::topological_sort;
use crate::{ast, common::*, context::Context, virdant_error};
use super::*;

#[salsa::query_group(CheckQStorage)]
pub trait CheckQ: typecheck::TypecheckQ {
    fn check(&self) -> VirdantResult<()>;
}

fn check(db: &dyn CheckQ) -> VirdantResult<()> {
    let package_ids = db.packages();

    check_no_import_cycles(db)?;

    for package_id in package_ids {
        check_ast_ok(db, package_id.clone())?;
        check_all_imported_packages_exist(db, package_id.clone())?;
        check_no_dup_imports(db, package_id.clone())?;

        for item_id in db.package_items(package_id.clone())? {
            check_item_no_dup_names(db, item_id.clone())?;
            check_all_dep_items_exist(db, item_id.clone())?;
        }

        for moddef_id in db.package_moddefs(package_id)? {
            check_all_targets_uniquely_driven(db, moddef_id.clone())?;
            check_wires_typecheck(db, moddef_id.clone())?;
            check_wires_correct_wiretype(db, moddef_id.clone())?;
            check_clocks_typecheck(db, moddef_id.clone())?;
            check_no_reads_from_sinks(db, moddef_id.clone())?;
        }
    }

    Ok(())
}

fn check_no_import_cycles(_db: &dyn CheckQ) -> VirdantResult<()> {
    eprintln!("SKIP check_no_import_cycles");
    Ok(())
}

fn check_ast_ok(db: &dyn CheckQ, package_id: PackageId) -> VirdantResult<()> {
    db.package_ast(package_id)?;
    Ok(())
}

fn check_all_imported_packages_exist(db: &dyn CheckQ, package_id: PackageId) -> VirdantResult<()> {
    let mut errors = ErrorReport::new();

    let all_packages = db.packages();
    for import_package_id in db.package_imports(package_id)? {
        if !all_packages.contains(&import_package_id) {
            errors.add(VirdantError::Other("Package doesn't exist".to_owned()));
        }
    }

    errors.check()?;
    Ok(())
}

fn check_no_dup_imports(db: &dyn CheckQ, package_id: PackageId) -> VirdantResult<()> {
    db.package_imports(package_id)?;
    Ok(())
}

fn check_item_no_dup_names(db: &dyn CheckQ, item_id: ItemId) -> VirdantResult<()> {
    db.item_elements(item_id)
        .map_err(|err| VirdantError::Other("Failed Check: Item No Dup Names".into()).because(err))?;
    Ok(())
}

fn check_all_dep_items_exist(db: &dyn CheckQ, item_id: ItemId) -> VirdantResult<()> {
    db.item_dependencies(item_id)
        .map_err(|err| VirdantError::Other("Failed Check: All Dep Items Exist".into()).because(err))?;
    Ok(())
}

fn check_all_targets_uniquely_driven(_db: &dyn CheckQ, _moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_all_targets_uniquely_driven");
    Ok(())
}

fn check_wires_typecheck(db: &dyn CheckQ, moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_wires_typecheck");
    let moddef_ast = db.moddef_ast(moddef_id.clone())?;


    for decl  in &moddef_ast.decls {
        if let ast::Decl::Wire(ast::Wire(target, _wire_type, expr)) = decl {
            let target: PathId = db.resolve_path(moddef_id.clone(), target.clone())?;
            let component_id = db.resolve_component(moddef_id.clone(), target)?;
            let target_typ = db.component_typ(component_id)?;
            let typed_expr = db.typecheck_expr(moddef_id.clone(), expr.clone(), target_typ, Context::empty())?;
            eprintln!("{typed_expr:?}");
        }
    }

    Ok(())
}

fn check_wires_correct_wiretype(_db: &dyn CheckQ, _moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_wires_correct_wiretype");
    Ok(())
}

fn resolve_element(db: &dyn CheckQ, item_id: ItemId, name: Ident) -> VirdantResult<ComponentId> {
    for element_id in db.item_elements(item_id.clone())? {
        if element_id.name() == name {
            return Ok(element_id);
        }
    }
    Err(virdant_error!("Unable to resolve element {name} in item {item_id}"))
}

fn follow_path(db: &dyn CheckQ, target: PathId, item_id: ItemId) -> VirdantResult<ComponentId> {
    eprintln!("follow_path({target}, {item_id})");
    let mut current_item_id = item_id;
    let mut remaining_path: Option<Path> = Some(target.as_path());
    let mut current_element: ComponentId = resolve_element(db, current_item_id.clone(), target.as_path().head())?;

    while let Some(path) = remaining_path {
        current_element = resolve_element(db, current_item_id.clone(), path.head())?;
        remaining_path = path.tail();
        eprintln!("  remaining path is {remaining_path:?} and current_element = {current_element}");
    }

    eprintln!("  result = {current_element}");
    Ok(current_element)
}

fn check_clocks_typecheck(_db: &dyn CheckQ, _moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_clocks_typecheck");
    Ok(())
}

fn check_no_reads_from_sinks(_db: &dyn CheckQ, _moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_no_reads_from_sinks");
    Ok(())
}
