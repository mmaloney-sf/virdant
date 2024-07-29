pub mod parse;
pub mod error;
pub mod id;

mod ready;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use ready::Ready;
use error::VirErr;
use error::VirErrs;
use id::*;
use parse::Ast;

/// A [`Virdant`] is a context type for manipulating Virdant designs.
/// Call [`check()`](Virdant::check) to get a list of errors in a design.
pub struct Virdant<'a> {
    sources: HashMap<Id<Package>, std::path::PathBuf>,
    asts: Ready<HashMap<Id<Package>, Ast<'a>>>,
    errors: VirErrs,
}

impl<'a> Virdant<'a> {
    pub fn new() -> Virdant<'a> {
        Virdant {
            sources: HashMap::new(),
            asts: Ready::new(),
            errors: VirErrs::new(),
        }
    }

    pub fn add_package_source<S, P>(&mut self, package: S, path: P) 
        where 
            S: Into<String>, 
            P: Into<std::path::PathBuf> {
        let id: Id<Package> = Id::from(package.into());
        self.sources.insert(id, path.into());
    }

    pub fn check(&mut self) -> Result<(), VirErrs> {
        if let Err(errs) = self.parse() {
            self.errors.extend(errs);
        }

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
        let mut imports: HashSet<Id<Package>> = HashSet::new();

        for import in self.package_imports(package) {
            if !imports.insert(import) {
                errors.add(VirErr::Other(format!("Duplicate import: {import}")));
            }
        }

        errors.check()
    }

    fn package_imports(&self, package: Id<Package>) -> Vec<Id<Package>> {
        let mut packages = vec![];
        let ast = &self.asts[&package];
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

    fn parse(&mut self) -> Result<(), VirErrs> {
        let mut errors = VirErrs::new();
        self.asts.set(HashMap::new());

        for package in self.packages() {
            let text = self.package_text(package).leak();
            match parse::parse_package(text) {
                Ok(ast) => {
                    self.asts.insert(package.clone(), ast);
                },
                Err(err) => {
                    errors.add(VirErr::Parse(err));
                },
            }
        }

        errors.check()
    }

    pub fn packages(&self) -> Vec<Id<Package>> {
        self.sources.keys().cloned().collect()
    }

    pub fn items(&self) -> Vec<Id<Item>> {
        let mut items = vec![];

        for (package, ast) in self.asts.iter() {
            for top_level_tree in ast.children() {
                if top_level_tree.is_item() {
                    let name = top_level_tree.name().unwrap();
                    let item: Id<Item> = format!("{package}::{name}").into();
                    items.push(item);
                }
            }
        }

        items
    }
}
