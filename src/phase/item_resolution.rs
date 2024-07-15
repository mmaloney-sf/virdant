use std::collections::HashSet;

use crate::common::*;
use crate::ast;
use super::*;

#[salsa::query_group(ItemResolutionQStorage)]
pub trait ItemResolutionQ: imports::ImportsQ {
    fn items(&self, package: PackageId) -> VirdantResult<Vec<ItemId>>;

    fn moddefs(&self, package: PackageId) -> VirdantResult<Vec<ModDefId>>;

    fn item(&self, item: Path, from: PackageId) -> VirdantResult<ItemId>;
}

fn items(db: &dyn ItemResolutionQ, package: PackageId) -> VirdantResult<Vec<ItemId>> {
    let mut items = vec![];
    let mut item_names = HashSet::new();
    let mut errors = ErrorReport::new();
    let package_ast = db.package_ast(package.clone())?;
    let package_path: Path = package.into();

    for item in &package_ast.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                let name = moddef_ast.name.clone();
                let moddef: ModDefId = package_path.join(&name.as_path()).into();
                items.push(ItemId::ModDef(moddef));
                if !item_names.insert(name.clone()) {
                    errors.add(VirdantError::Other(format!("Duplicate item name in package {package_path}: {name}")))
                }
            },
            ast::Item::StructDef(structdef_ast) => {
                let name = structdef_ast.name.clone();
                let structdef: StructDefId = package_path.join(&name.as_path()).into();
                items.push(ItemId::StructDef(structdef));
                if !item_names.insert(name.clone()) {
                    errors.add(VirdantError::Other(format!("Duplicate item name in package {package_path}: {name}")))
                }
            },
            ast::Item::UnionDef(uniondef_ast) => {
                let name = uniondef_ast.name.clone();
                let uniondef: UnionDefId = package_path.join(&name.as_path()).into();
                items.push(ItemId::UnionDef(uniondef));
                if !item_names.insert(name.clone()) {
                    errors.add(VirdantError::Other(format!("Duplicate item name in package {package_path}: {name}")))
                }
            },
            ast::Item::PortDef(_) => todo!(),

        }
    }
    Ok(items)
}

fn moddefs(db: &dyn ItemResolutionQ, package: PackageId) -> VirdantResult<Vec<ModDefId>> {
    let moddefs = db.items(package)?
        .into_iter()
        .filter_map(|item| {
            if let ItemId::ModDef(moddef) = item {
                Some(moddef)
            } else {
                None
            }
        })
        .collect();
    Ok(moddefs)
}

fn item(db: &dyn ItemResolutionQ, path: Path, from: PackageId) -> VirdantResult<ItemId> {
    let imported_packages = db.package_imports(from.clone())?;
    let path_package = PackageId::from(path.head().as_path());

    if imported_packages.contains(&PackageId::from(path.head().as_path())) {
        // try to interpret the path as imported_package.rest.of.path
        for item in db.items(path_package)? {
            let item_path: Path = item.clone().into();
            if item_path == path {
                return Ok(item);
            }
        }
    } else {
        // otherwise, treat it as a local path
        for package in db.packages() {
            for item in db.items(package)? {
                let item_path: Path = item.clone().into();
                if item_path == Path::from(from.clone()).join(&path) {
                    return Ok(item);
                }
            }
        }
    }

    Err(VirdantError::Other(format!("Could not resolve item for path: {path}")))
}
