//use crate::topological_sort::topological_sort;
use crate::common::*;
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
            check_all_dep_items_exist(db, item_id)?;
        }

        for _moddef_id in db.package_moddefs(package_id)? {
        }
    }

    Ok(())
}

fn check_no_import_cycles(_db: &dyn CheckQ) -> VirdantResult<()> { eprintln!("SKIP check_no_import_cycles"); Ok(())
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
