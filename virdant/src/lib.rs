/// Utilities for parsing a Virdant source file.
///
/// [`parse_package()`](parse::parse_package) is used to parse a package.
/// This results in a [`ParseTree`](parse::ParseTree) object (or a [`ParseError`](parse::ParseError) on failure).
pub mod parse;
pub mod error;
pub mod id;

mod ready;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use ready::Ready;
use error::VirErr;
use error::VirErrs;
use id::*;


pub struct Virdant<'a> {
    sources: HashMap<Id<Package>, std::path::PathBuf>,
    parse_trees: Ready<HashMap<Id<Package>, parse::ParseTree<'a>>>,
    errors: VirErrs,
}

impl<'a> Virdant<'a> {
    pub fn new() -> Virdant<'a> {
        Virdant {
            sources: HashMap::new(),
            parse_trees: Ready::new(),
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

        self.errors.clone().check()
    }

    fn package_text(&self, package: Id<Package>) -> String {
        let path = self.sources.get(&package).unwrap();
        std::fs::read_to_string(path).unwrap()
    }

    fn parse(&mut self) -> Result<(), VirErrs> {
        let mut errors = VirErrs::new();
        self.parse_trees.set(HashMap::new());

        for package in self.packages() {
            let text = self.package_text(package).leak();
            match parse::parse_package(text) {
                Ok(parse_tree) => {
                    self.parse_trees.insert(package.clone(), parse_tree);
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

        for (package, parse_tree) in self.parse_trees.iter() {
            for top_level_tree in parse_tree.children() {
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
