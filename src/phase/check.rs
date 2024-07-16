use crate::topological_sort::topological_sort;
use crate::common::*;
use super::*;

#[salsa::query_group(CheckQStorage)]
pub trait CheckQ: typecheck::TypecheckQ {
    fn check(&self) -> VirdantResult<()>;
}

fn check_package(_db: &dyn CheckQ, package: PackageId) -> VirdantResult<()> {
    eprintln!("Checking package {package}");

//    for item in db.package_items(package)? {
//    }

    Ok(())
}

fn check(db: &dyn CheckQ) -> VirdantResult<()> {
    let mut errors = ErrorReport::new();
    let packages = db.packages();
    let mut package_dependencies = HashMap::new();

    for package in packages {
        let dependencies = db.package_imports(package.clone())?;
        package_dependencies.insert(package, dependencies);
    }

    let sorted_packages = topological_sort(&package_dependencies)?;

    for package in sorted_packages {
        errors.add_on_err(check_package(db, package));
    }

    errors.check()?;
    Ok(())
}
