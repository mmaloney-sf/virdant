use crate::topological_sort::topological_sort;
use crate::common::*;
use super::*;

#[salsa::query_group(CheckQStorage)]
pub trait CheckQ: typecheck::TypecheckQ {
    fn check_package(&self, package: PackageId) -> VirdantResult<()>;

    fn check(&self) -> VirdantResult<()>;
}

fn check_package(db: &dyn CheckQ, package: PackageId) -> VirdantResult<()> {
    eprintln!("Checking package {package}");

    Ok(())
}

fn check(db: &dyn CheckQ) -> VirdantResult<()> {
    let packages = db.packages();
    let mut package_dependencies = HashMap::new();

    for package in packages {
        let dependencies = db.package_imports(package.clone())?;
        package_dependencies.insert(package, dependencies);
    }

    let sorted_packages = topological_sort(&package_dependencies)?;

    for package in sorted_packages {
        db.check_package(package)?;
    }

    Ok(())
}
