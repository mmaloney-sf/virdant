pub mod parse;
pub mod error;
pub mod id;

mod table;
mod ready;

#[cfg(test)]
mod tests;

use indexmap::IndexSet;
use ready::Ready;
use error::VirErr;
use error::VirErrs;
use id::*;
use parse::Ast;
use std::hash::Hash;
use table::Table;


/// A [`Virdant`] is a context type for manipulating Virdant designs.
/// Call [`check()`](Virdant::check) to get a list of errors in a design.
#[derive(Default)]
pub struct Virdant<'a> {
    errors: VirErrs,

    packages: Table<Package, PackageInfo<'a>>,
    items: Table<Item, ItemInfo<'a>>,
}

#[derive(Default, Clone)]
struct PackageInfo<'a> {
    name: String,
    source: std::path::PathBuf,
    ast: Ready<Ast<'a>>,
}

#[derive(Default, Clone)]
struct ItemInfo<'a> {
    name: String,
    package: Ready<Id<Package>>,
    ast: Ready<Ast<'a>>,
    kind: Ready<ItemKind>,
}

////////////////////////////////////////////////////////////////////////////////
// Public Virdant<'a> API
////////////////////////////////////////////////////////////////////////////////

impl<'a> Virdant<'a> {
    pub fn new() -> Virdant<'a> {
        Virdant::default()
    }

    pub fn add_package_source<S, P>(&mut self, package: S, path: P)
        where
            S: Into<String>,
            P: Into<std::path::PathBuf> {
        let package_name = package.into();
        let package_id: Id<Package> = Id::from(package_name.clone());
        let package_info = self.packages.register(package_id);
        package_info.name = package_name;
        package_info.source = path.into();
    }

    pub fn check(&mut self) -> Result<(), VirErrs> {
        self.init_asts();

        for package in self.packages() {
            if let Err(errs) = self.all_imported_packages_exist(package) {
                self.errors.extend(errs)
            }

            if let Err(errs) = self.no_duplicate_imports(package) {
                self.errors.extend(errs)
            }
        }

        self.errors.clone().check()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Analyses
////////////////////////////////////////////////////////////////////////////////

impl<'a> Virdant<'a> {
    fn init_asts(&mut self) {
        for package in self.packages() {
            let text: &'a str = self.package_text(package).leak();
            let result: Result<Ast<'a>, _> = parse::parse_package(text);
            match result {
                Ok(package_ast) => {
                    let package_info = self.packages.get_mut(package).unwrap();
                    package_info.ast.set(package_ast.clone());
                    self.init_item_asts(package);
                },
                Err(err) => self.errors.add(VirErr::Parse(err)),
            }
        }
    }

    fn init_item_asts(&mut self, package: Id<Package>) {
        let package_ast = &self.packages[package].ast.unwrap();
        for node in package_ast.children() {
            if node.is_item() {
                let item_name = node.name().unwrap();
                let item: Id<Item> = Id::from(format!("{package}::{item_name}"));

                if self.items.is_registered(item) {
                    self.errors.add(VirErr::DupItem(item));
                }

                let item_info = self.items.register(item);
                let kind = node.item_kind().unwrap();

                item_info.name = item_name.to_string();
                item_info.kind.set(kind);
                item_info.package.set(package);
                item_info.ast.set(node);
            }
        }
    }

    #[cfg(test)]
    fn items(&self) -> Vec<Id<Item>> {
        self.items.keys().cloned().collect()
    }

    #[cfg(test)]
    fn moddefs(&self) -> Vec<Id<ModDef>> {
        let mut results = vec![];
        for item in self.items.keys() {
            let item_ast = &self.items[*item].ast.unwrap();
            if let Some(ItemKind::ModDef) = item_ast.item_kind() {
                results.push(item.cast());
            }
        }
        results
    }
}


////////////////////////////////////////////////////////////////////////////////
// Internal checks
////////////////////////////////////////////////////////////////////////////////

impl<'a> Virdant<'a> {
    fn packages(&self) -> Vec<Id<Package>> {
        self.packages.keys().cloned().collect()
    }

    fn all_imported_packages_exist(&mut self, package: Id<Package>) -> Result<(), VirErrs> {
        let mut errors = VirErrs::new();
        let packages = self.packages();
        for imported_package in self.package_imports(package) {
            if !packages.contains(&imported_package) {
                errors.add(VirErr::CantImport(imported_package));
            }
        }
        errors.check()
    }

    fn no_duplicate_imports(&mut self, package: Id<Package>) -> Result<(), VirErrs> {
        let mut errors = VirErrs::new();
        let mut imports: IndexSet<Id<Package>> = IndexSet::new();

        for import in self.package_imports(package) {
            if !imports.insert(import) {
                errors.add(VirErr::DupImport(import));
            }
        }

        errors.check()
    }

    fn package_imports(&self, package: Id<Package>) -> Vec<Id<Package>> {
        let mut packages = vec![];
        if let Ok(ast) = &self.packages[package].ast.get() {
            for node in ast.children() {
                if node.is_import() {
                    let import_package = Id::new(node.package().unwrap());
                    packages.push(import_package);
                }
            }
        }

        packages
    }

    fn package_text(&self, package: Id<Package>) -> String {
        let path = &self.packages[package].source;
        std::fs::read_to_string(path).unwrap()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ItemKind {
    ModDef,
    UnionDef,
    StructDef,
    PortDef,
}
