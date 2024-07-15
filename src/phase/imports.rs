use crate::ast;
use crate::common::*;
use super::*;

use std::collections::HashSet;

#[salsa::query_group(ImportsQStorage)]
pub trait ImportsQ: astq::AstQ {
    fn package_imports(&self, package: PackageId) -> VirdantResult<Vec<PackageId>>;

//   fn imports(&self) -> VirdantResult<Vec<PackageId>>;
}

fn package_imports(db: &dyn ImportsQ, package: PackageId) -> VirdantResult<Vec<PackageId>> {
    let package_ast = db.package_ast(package)?;
    let mut errors = ErrorReport::new();
    let mut packages = HashSet::new();
    for ast::PackageImport(package_name) in &package_ast.imports {
        if !packages.insert(package_name.as_path().into()) {
            errors.add(VirdantError::Other(format!("Duplicate import: {package_name}")));
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
