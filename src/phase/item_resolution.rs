use std::collections::HashSet;

use crate::common::*;
use crate::ast;
use super::*;
use super::astq;

#[salsa::query_group(ItemResolutionQStorage)]
pub trait ItemResolutionQ: astq::AstQ {
    fn items(&self, package: Package) -> VirdantResult<Vec<Item>>;

    fn moddefs(&self, package: Package) -> VirdantResult<Vec<ModDef>>;

    fn item(&self, item: Path) -> VirdantResult<Item>;
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
            ast::Item::StructTypeDef(structdef_ast) => {
                let name = structdef_ast.name.clone();
                let structdef: StructDef = package_path.join(&name.as_path()).into();
                items.push(Item::StructDef(structdef));
                if !item_names.insert(name.clone()) {
                    errors.add(VirdantError::Other(format!("Duplicate item name in package {package_path}: {name}")))
                }
            },
            ast::Item::AltTypeDef(uniondef_ast) => {
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

fn item(db: &dyn ItemResolutionQ, item: Path) -> VirdantResult<Item> {
    for package in db.packages() {
        for package_item in db.items(package)? {
            let item_path: Path = package_item.clone().into();
            if item_path == item {
                return Ok(package_item);
            }
        }
    }
    Err(VirdantError::Other(format!("Could not resolve item for path: {item}")))
}
