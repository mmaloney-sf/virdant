//use crate::topological_sort::topological_sort;
use crate::{ast, common::*, context::Context, virdant_error, virdant_error_at};
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
            errors.add(virdant_error!("Package doesn't exist"));
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
        .map_err(|err| virdant_error!("Failed Check: Item No Dup Names").because(err))?;
    Ok(())
}

fn check_all_dep_items_exist(db: &dyn CheckQ, item_id: ItemId) -> VirdantResult<()> {
    db.item_dependencies(item_id)
        .map_err(|err| virdant_error!("Failed Check: All Dep Items Exist").because(err))?;
    Ok(())
}

fn check_all_targets_uniquely_driven(_db: &dyn CheckQ, _moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_all_targets_uniquely_driven");
    Ok(())
}

fn check_wires_typecheck(db: &dyn CheckQ, moddef_id: ModDefId) -> VirdantResult<()> {
    let mut errors = ErrorReport::new();
    let moddef_ast = db.moddef_ast(moddef_id.clone())?;

    for decl in &moddef_ast.decls {
        if let ast::Decl::Wire(wire) = decl {
            let ast::Wire(target, _wire_type, expr) = wire.as_ref();
            let element_id = db.resolve_component_by_path(moddef_id.clone(), target.clone())?;
            let target_typ = db.component_typ(element_id)?;
            let typed_expr = db.typecheck_expr(moddef_id.clone(), expr.clone(), target_typ, Context::empty());
            if let Err(e) = typed_expr {
                // errors.add(e);
                let span = db.span(wire.span());
                errors.add(virdant_error_at!("Typecheck failed", span).because(e));
            }
        }
    }

    errors.check()?;
    Ok(())
}

fn check_wires_correct_wiretype(_db: &dyn CheckQ, _moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_wires_correct_wiretype");
    Ok(())
}

fn check_clocks_typecheck(_db: &dyn CheckQ, _moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_clocks_typecheck");
    Ok(())
}

fn check_no_reads_from_sinks(_db: &dyn CheckQ, _moddef_id: ModDefId) -> VirdantResult<()> {
    eprintln!("SKIP check_no_reads_from_sinks");
    Ok(())
}
