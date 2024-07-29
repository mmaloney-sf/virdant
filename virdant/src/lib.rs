pub mod parse;
pub mod error;
pub mod id;

mod ready;

#[cfg(test)]
mod tests;

use indexmap::{IndexMap, IndexSet};
use ready::Ready;
use error::VirErr;
use error::VirErrs;
use id::*;
use parse::Ast;


/// A [`Virdant`] is a context type for manipulating Virdant designs.
/// Call [`check()`](Virdant::check) to get a list of errors in a design.
#[derive(Default)]
pub struct Virdant<'a> {
    sources: IndexMap<Id<Package>, std::path::PathBuf>,
    errors: VirErrs,

    package_asts: Ready<IndexMap<Id<Package>, Ast<'a>>>,
    items: Ready<Vec<Id<Item>>>,
    item_asts: Ready<IndexMap<Id<Item>, Result<Ast<'a>, VirErr>>>,
    item_kinds: Ready<IndexMap<Id<Item>, Result<ItemKind, VirErr>>>,
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
        let id: Id<Package> = Id::from(package.into());
        self.sources.insert(id, path.into());
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
        self.package_asts.set(IndexMap::new());
        self.items.set(Vec::new());
        self.item_asts.set(IndexMap::new());
        self.item_kinds.set(IndexMap::new());

        for package in self.packages() {
            let text: &'a str = self.package_text(package).leak();
            let result: Result<Ast<'a>, _> = parse::parse_package(text);
            match result {
                Ok(package_ast) => {
                    self.package_asts.insert(package.clone(), package_ast.clone());
                    self.init_item_asts(package);
                },
                Err(err) => {
                    self.errors.add(VirErr::Parse(err));
                },
            }
        }
    }

    fn init_item_asts(&mut self, package: Id<Package>) {
        let package_ast = &self.package_asts[&package];
        for node in package_ast.children() {
            if node.is_item() {
                let item_name = node.name().unwrap();
                let item: Id<Item> = Id::from(format!("{package}::{item_name}"));

                if let Some(_prev_node) = self.item_asts.insert(item, Ok(node.clone())) {
                    self.item_asts.insert(item, Err(VirErr::Other("Duplicate item".to_string())));
                } else {
                    self.items.push(item);
                }

                let kind = node.item_kind().unwrap();
                if let Some(prev_kind) = self.item_kinds.insert(item, Ok(kind)) {
                    if let Ok(true) = prev_kind.map(|other_kind| other_kind != kind) {
                        self.item_asts.insert(item, Err(VirErr::Other("Duplicate item with differing item kinds".to_string())));
                    }
                }
            }
        }
    }

    #[cfg(test)]
    fn items(&self) -> Vec<Id<Item>> {
        self.items.iter().cloned().collect()
    }

    #[cfg(test)]
    fn moddefs(&self) -> Vec<Id<ModDef>> {
        let mut results = vec![];
        for item in self.items.iter() {
            let item_ast = &self.item_asts[item];
            if let Ok(Some(ItemKind::ModDef)) = item_ast.as_ref().map(|ast| ast.item_kind()) {
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
    fn all_imported_packages_exist(&mut self, package: Id<Package>) -> Result<(), VirErrs> {
        let mut errors = VirErrs::new();
        let packages = self.packages();
        for imported_package in self.package_imports(package) {
            if !packages.contains(&imported_package) {
                errors.add(VirErr::Other(format!("Imported package does not exist: {imported_package}")));
            }
        }
        errors.check()
    }

    fn no_duplicate_imports(&mut self, package: Id<Package>) -> Result<(), VirErrs> {
        let mut errors = VirErrs::new();
        let mut imports: IndexSet<Id<Package>> = IndexSet::new();

        for import in self.package_imports(package) {
            if !imports.insert(import) {
                errors.add(VirErr::Other(format!("Duplicate import: {import}")));
            }
        }

        errors.check()
    }

    fn package_imports(&self, package: Id<Package>) -> Vec<Id<Package>> {
        let mut packages = vec![];
        let ast = &self.package_asts[&package];
        for node in ast.children() {
            if node.is_import() {
                let import_package = Id::new(node.package().unwrap());
                packages.push(import_package);
            }
        }

        packages
    }

    fn package_text(&self, package: Id<Package>) -> String {
        let path = self.sources.get(&package).unwrap();
        std::fs::read_to_string(path).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Helpers
////////////////////////////////////////////////////////////////////////////////

impl<'a> Virdant<'a> {
    pub fn packages(&self) -> Vec<Id<Package>> {
        self.sources.keys().cloned().collect()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ItemKind {
    ModDef,
    UnionDef,
    StructDef,
    PortDef,
}
