pub mod parse;
pub mod error;
pub mod id;

mod table;
mod ready;

#[cfg(test)]
mod tests;

use indexmap::IndexMap;
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
    pub fn new<S, P>(sources: &[(S, P)]) -> Virdant<'a>
        where
            S: AsRef<str>,
            P: AsRef<std::path::Path> {
        let mut virdant = Virdant::default();

        let sources: IndexMap<String, std::path::PathBuf> = sources
            .into_iter()
            .map(|(s, p)| {
                let s: String = s.as_ref().to_owned();
                let p: std::path::PathBuf = p.as_ref().to_owned();
                (s, p)
            })
            .collect();

        virdant.register_packages(sources);
        virdant
    }

    pub fn check(&mut self) -> Result<(), VirErrs> {
        self.init_package_asts();
        self.register_items();

        let packages: Vec<_> = self.packages.keys().cloned().collect();
        for package in packages {
            if let Err(errs) = self.check_all_imported_packages_exist(package) {
                self.errors.extend(errs)
            }

            if let Err(errs) = self.check_no_duplicate_imports(package) {
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
    fn register_packages(&mut self, sources: IndexMap<String, std::path::PathBuf>) {
        for (package_name, package_path) in sources {
            let package: Id<Package> = Id::new(package_name.clone());
            let package_info = self.packages.register(package);
            package_info.name = package_name;
            package_info.source = package_path;
        }
    }

    fn init_package_asts(&mut self) {
        let packages: Vec<_> = self.packages.keys().cloned().collect();
        for package in packages {
            let text: &'a str = self.package_text(package).leak();
            let result: Result<Ast<'a>, _> = parse::parse_package(text);
            match result {
                Ok(package_ast) => {
                    let package_info = &mut self.packages[package];
                    package_info.ast.set(package_ast.clone());
                },
                Err(err) => self.errors.add(VirErr::Parse(err)),
            }
        }
    }

    fn register_items(&mut self) {
        for package in self.packages.keys().cloned() {
            if let Ok(package_ast) = &self.packages[package].ast.get() {
                for node in package_ast.children() {
                    if node.is_item() {
                        let item_name = node.name().unwrap();
                        let item: Id<Item> = Id::new(format!("{package}::{item_name}"));

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
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Resolution
////////////////////////////////////////////////////////////////////////////////

impl<'a> Virdant<'a> {
    fn resolve_package(&self, package_name: &str) -> Option<Id<Package>> {
        for (package, package_info) in self.packages.iter() {
            if package_name == package_info.name {
                return Some(*package);
            }
        }
        None
    }
}


////////////////////////////////////////////////////////////////////////////////
// Internal checks
////////////////////////////////////////////////////////////////////////////////

impl<'a> Virdant<'a> {
    fn check_all_imported_packages_exist(&mut self, package: Id<Package>) -> Result<(), VirErrs> {
        let mut errors = VirErrs::new();
        for imported_package_name in self.package_imports(package) {
            let imported_package = self.resolve_package(&imported_package_name);
            if imported_package.is_none() {
                errors.add(VirErr::CantImport(imported_package_name));
            }
        }
        errors.check()
    }

    fn check_no_duplicate_imports(&mut self, package: Id<Package>) -> Result<(), VirErrs> {
        let mut errors = VirErrs::new();
        let mut imports: IndexSet<String> = IndexSet::new();

        for import in self.package_imports(package) {
            if !imports.insert(import.clone()) {
                errors.add(VirErr::DupImport(import));
            }
        }

        errors.check()
    }

    fn package_imports(&self, package: Id<Package>) -> Vec<String> {
        let mut packages = vec![];
        if let Ok(ast) = &self.packages[package].ast.get() {
            for node in ast.children() {
                if node.is_import() {
                    packages.push(node.package().unwrap().to_string());
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


////////////////////////////////////////////////////////////////////////////////
// For testing
////////////////////////////////////////////////////////////////////////////////

impl<'a> Virdant<'a> {
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

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ItemKind {
    ModDef,
    UnionDef,
    StructDef,
    PortDef,
}
