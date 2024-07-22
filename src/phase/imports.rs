use crate::ast;
use crate::common::*;
use crate::virdant_error_at;
use super::*;

use std::collections::HashSet;

#[salsa::query_group(ImportsQStorage)]
pub trait ImportsQ: astq::AstQ {
    fn package_imports(&self, package_id: PackageId) -> VirdantResult<Vec<PackageId>>;

//   fn imports(&self) -> VirdantResult<Vec<PackageId>>;
}

fn package_imports(db: &dyn ImportsQ, package_id: PackageId) -> VirdantResult<Vec<PackageId>> {
    let package_ast = db.package_ast(package_id)?;
    let mut errors = ErrorReport::new();
    let mut packages = HashSet::new();

    for package_import in &package_ast.imports {
        let ast::PackageImport(package_name) = package_import.as_ref();
        let imported_package_id = PackageId::from_ident(package_name.clone());
        if !packages.insert(imported_package_id) {
            let span = db.span(package_import.span());
            errors.add(virdant_error_at!("Duplicate import: {package_name}", span));
        }
    }
    errors.check()?;

    Ok(packages.into_iter().collect())
}

/*
fn imports(db: &dyn ImportsQ) -> VirdantResult<Vec<PackageId>> {
    let mut stack = vec![];
    let mut ordered_imports = vec![];

    for package in db.packages() {
        if !stack.contains(&package) {
            stack.push(package.clone());
        }

        for dep_package in db.package_imports(package) {
            if !stack.contains(&package) {
                stack.push(package.clone());
            }
        }
    }

    Ok(ordered_imports)
}
*/
