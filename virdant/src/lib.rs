/// Utilities for parsing a Virdant source file.
///
/// [`parse_package()`](parse::parse_package) is used to parse a package.
/// This results in a [`ParseTree`](parse::ParseTree) object (or a [`ParseError`](parse::ParseError) on failure).
pub mod parse;
pub mod error;

mod ready;

#[cfg(test)]
mod tests;

use internment::Intern;
use std::collections::HashMap;
use ready::Ready;
use error::VirErr;
use error::VirErrs;

pub struct Virdant<'a> {
    sources: HashMap<Intern<String>, std::path::PathBuf>,
    parse_trees: Ready<HashMap<Intern<String>, parse::ParseTree<'a>>>,
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
        self.sources.insert(Intern::new(package.into()), path.into());
    }

    pub fn check(&mut self) -> Result<(), VirErrs> {
        if let Err(errs) = self.parse() {
            self.errors.extend(errs);
        }

        self.errors.clone().check()
    }

    fn package_text(&self, package: Intern<String>) -> String {
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

    pub fn packages(&self) -> Vec<Intern<String>> {
        self.sources.keys().cloned().collect()
    }

    pub fn items(&self) -> Vec<Intern<String>> {
        let mut items = vec![];

        for (package, parse_tree) in self.parse_trees.iter() {
            for top_level_tree in parse_tree.children() {
                if top_level_tree.is_item() {
                    let name = top_level_tree.name().unwrap();
                    items.push(Intern::new(format!("{package}::{name}")));
                }
            }
        }

        items
    }
}
