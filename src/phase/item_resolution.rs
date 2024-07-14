use std::collections::HashSet;

use crate::common::*;
use crate::ast;
use super::*;
use super::astq;

#[salsa::query_group(ItemResolutionQStorage)]
pub trait ItemResolutionQ: astq::AstQ {
    fn items(&self, package: Package) -> VirdantResult<Vec<Item>>;

    fn moddefs(&self, package: Package) -> VirdantResult<Vec<ModDef>>;

    fn item(&self, item: Path, from: Package) -> VirdantResult<Item>;
}

fn items(db: &dyn ItemResolutionQ, package: Package) -> VirdantResult<Vec<Item>> {
    let mut items = vec![];
    let mut item_names = HashSet::new();
    let mut errors = ErrorReport::new();
    let package_ast = db.package_ast(package.clone())?;
    let package_path: Path = package.into();

    for item in &package_ast.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                let name = moddef_ast.name.clone();
                let moddef: ModDef = package_path.join(&name.as_path()).into();
                items.push(Item::ModDef(moddef));
                if !item_names.insert(name.clone()) {
                    errors.add(VirdantError::Other(format!("Duplicate item name in package {package_path}: {name}")))
                }
            },
            ast::Item::StructDef(structdef_ast) => {
                let name = structdef_ast.name.clone();
                let structdef: StructDef = package_path.join(&name.as_path()).into();
                items.push(Item::StructDef(structdef));
                if !item_names.insert(name.clone()) {
                    errors.add(VirdantError::Other(format!("Duplicate item name in package {package_path}: {name}")))
                }
            },
            ast::Item::UnionDef(uniondef_ast) => {
                let name = uniondef_ast.name.clone();
                let uniondef: UnionDef = package_path.join(&name.as_path()).into();
                items.push(Item::UnionDef(uniondef));
                if !item_names.insert(name.clone()) {
                    errors.add(VirdantError::Other(format!("Duplicate item name in package {package_path}: {name}")))
                }
            },
        }
    }
    Ok(items)
}

fn moddefs(db: &dyn ItemResolutionQ, package: Package) -> VirdantResult<Vec<ModDef>> {
    let moddefs = db.items(package)?
        .into_iter()
        .filter_map(|item| {
            if let Item::ModDef(moddef) = item {
                Some(moddef)
            } else {
                None
            }
        })
        .collect();
    Ok(moddefs)
}

fn item(db: &dyn ItemResolutionQ, path: Path, from: Package) -> VirdantResult<Item> {
    let imported_packages = db.imports(from.clone())?;
    let path_package = Package::from(path.head().as_path());

    if imported_packages.contains(&Package::from(path.head().as_path())) {
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
