use std::collections::HashSet;

use crate::common::*;
use crate::ast;
use crate::virdant_error;
use super::*;

#[salsa::query_group(ItemResolutionQStorage)]
pub trait ItemResolutionQ: imports::ImportsQ + item_namespace::ItemNamespaceQ {
    fn items(&self) -> VirdantResult<Vec<ItemId>>;

    fn package_items(&self, package: PackageId) -> VirdantResult<Vec<ItemId>>;

    fn package_moddefs(&self, package: PackageId) -> VirdantResult<Vec<ModDefId>>;

    fn item(&self, item: QualIdent, package: PackageId) -> VirdantResult<ItemId>;

    fn moddef(&self, moddef: QualIdent, package: PackageId) -> VirdantResult<ModDefId>;
    fn portdef(&self, portdef: QualIdent, package: PackageId) -> VirdantResult<PortDefId>;

    fn resolve_package(&self, package: Ident) -> VirdantResult<PackageId>;
}

fn resolve_package(db: &dyn ItemResolutionQ, package_name: Ident) -> VirdantResult<PackageId> {
    for package in db.packages() {
        if package.name() == package_name {
            return Ok(package);
        }
    }
    Err(virdant_error!("Unknown package: {package_name}"))
}

fn items(db: &dyn ItemResolutionQ) -> VirdantResult<Vec<ItemId>> {
    let mut errors = ErrorReport::new();
    let mut items = vec![];
    for package in db.packages() {
        match db.package_items(package) {
            Ok(package_items) => items.extend(package_items),
            Err(err) => errors.add(err),
        }
    }
    errors.check()?;
    Ok(items)
}

fn package_items(db: &dyn ItemResolutionQ, package_id: PackageId) -> VirdantResult<Vec<ItemId>> {
    let mut items = vec![];
    let mut item_names = HashSet::new();
    let mut errors = ErrorReport::new();
    let package_ast = db.package_ast(package_id.clone())?;

    for item in &package_ast.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                let name = moddef_ast.name.clone();
                let moddef = ModDefId::from_ident(package_id.clone(), name.clone());
                items.push(ItemId::ModDef(moddef));
                if !item_names.insert(name.clone()) {
                    errors.add(virdant_error!("Duplicate item name in package {name}."))
                }
            },
            ast::Item::StructDef(structdef_ast) => {
                let name = structdef_ast.name.clone();
                let structdef = StructDefId::from_ident(package_id.clone(), name.clone());
                items.push(ItemId::StructDef(structdef));
                if !item_names.insert(name.clone()) {
                    errors.add(virdant_error!("Duplicate item name in package {name}"))
                }
            },
            ast::Item::UnionDef(uniondef_ast) => {
                let name = uniondef_ast.name.clone();
                let uniondef = UnionDefId::from_ident(package_id.clone(), name.clone());
                items.push(ItemId::UnionDef(uniondef));
                if !item_names.insert(name.clone()) {
                    errors.add(virdant_error!("Duplicate item name in package {name}"))
                }
            },
            ast::Item::PortDef(portdef_ast) => {
                let name = portdef_ast.name.clone();
                let portdef = PortDefId::from_ident(package_id.clone(), name.clone());
                items.push(ItemId::PortDef(portdef));
                if !item_names.insert(name.clone()) {
                    errors.add(virdant_error!("Duplicate item name in package {name}"))
                }
            },

        }
    }
    Ok(items)
}

fn package_moddefs(db: &dyn ItemResolutionQ, package: PackageId) -> VirdantResult<Vec<ModDefId>> {
    let moddefs = db.package_items(package)?
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

fn item(db: &dyn ItemResolutionQ, item: QualIdent, package_id: PackageId) -> VirdantResult<ItemId> {
    let imported_packages = db.package_imports(package_id.clone())?;

    let item_package_id = if let Some(namespace) = item.namespace() {
        db.resolve_package(namespace)?
    } else {
        package_id.clone()
    };

    if imported_packages.contains(&item_package_id) || item_package_id == package_id {
        for package_item in db.package_items(item_package_id.clone())? {
            if package_item.name() == item.name() {
                return Ok(package_item);
            }
        }
    }

    Err(virdant_error!("Could not resolve item: {item}"))
}

fn moddef(db: &dyn ItemResolutionQ, moddef: QualIdent, package_id: PackageId) -> VirdantResult<ModDefId> {
    if let ItemId::ModDef(moddef_id) = db.item(moddef.clone(), package_id)? {
        Ok(moddef_id)
    } else {
        Err(virdant_error!("Item {moddef} is not a mod def"))
    }
}

fn portdef(db: &dyn ItemResolutionQ, portdef: QualIdent, package_id: PackageId) -> VirdantResult<PortDefId> {
    if let ItemId::PortDef(portdef_id) = db.item(portdef.clone(), package_id)? {
        Ok(portdef_id)
    } else {
        Err(virdant_error!("Item {portdef} is not a mod def"))
    }
}
