pub mod parse;
pub mod error;
pub mod id;
pub mod types;

mod table;
mod ready;
mod cycle;

#[cfg(test)]
mod tests;

use cycle::detect_cycle;
use indexmap::IndexMap;
use indexmap::IndexSet;
use parse::QualIdent;
use ready::Ready;
use error::VirErr;
use error::VirErrs;
use id::*;
use parse::Ast;
use types::CtorSig;
use std::hash::Hash;
use table::Table;
use types::Type;


////////////////////////////////////////////////////////////////////////////////
// Types
////////////////////////////////////////////////////////////////////////////////

/// A [`Virdant`] is a context type for manipulating Virdant designs.
/// Call [`check()`](Virdant::check) to get a list of errors in a design.
#[derive(Default)]
pub struct Virdant {
    errors: VirErrs,

    packages: Table<Package, PackageInfo>,
    items: Table<Item, ItemInfo>,
    structdefs: Table<StructDef, StructDefInfo>,
    fields: Table<Field, FieldInfo>,
    uniondefs: Table<UnionDef, UnionDefInfo>,
    ctors: Table<Ctor, CtorInfo>,
}

#[derive(Default, Clone, Debug)]
struct PackageInfo {
    name: String,
    source: std::path::PathBuf,
    ast: Ready<Ast<'static>>,
}

#[derive(Default, Clone, Debug)]
struct ItemInfo {
    name: String,
    package: Ready<Id<Package>>,
    ast: Ready<Ast<'static>>,
    kind: Ready<ItemKind>,
    deps: Ready<Vec<Id<Item>>>,
}

#[derive(Default, Clone, Debug)]
struct StructDefInfo {
    item: Ready<Id<Item>>,
    fields: Ready<Vec<Id<Field>>>,
}

#[derive(Default, Clone, Debug)]
struct FieldInfo {
    structdef: Ready<Id<StructDef>>,
    name: String,
    typ: Ready<Type>,
}

#[derive(Default, Clone, Debug)]
struct UnionDefInfo {
    item: Ready<Id<Item>>,
    ctors: Ready<Vec<Id<Ctor>>>,
}

#[derive(Default, Clone, Debug)]
struct CtorInfo {
    uniondef: Ready<Id<UnionDef>>,
    name: String,
    sig: Ready<CtorSig>,
}

////////////////////////////////////////////////////////////////////////////////
// Public Virdant API
////////////////////////////////////////////////////////////////////////////////

impl Virdant {
    pub fn new<S, P>(sources: &[(S, P)]) -> Virdant
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

        let items: Vec<_> = self.items.keys().cloned().collect();
        for item in items {
            let item_deps = self.item_deps(item).clone();
            let item_info = self.items.get_mut(item).unwrap();
            item_info.deps.set(item_deps);
        }

        self.check_no_item_dep_cycles();
        self.register_structdefs();
        self.register_uniondefs();

        self.errors.clone().check()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Packages and Items
////////////////////////////////////////////////////////////////////////////////

impl Virdant {
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
            match self.package_text(package) {
                Err(err) => self.errors.add(err),
                Ok(text) => {
                    let text: &'static str = text.leak(); // TODO
                    let result: Result<Ast, _> = parse::parse_package(text);
                    match result {
                        Ok(package_ast) => {
                            let package_info = &mut self.packages[package];
                            package_info.ast.set(package_ast.clone());
                        },
                        Err(err) => self.errors.add(VirErr::Parse(err)),
                    }
                }
            }
        }
    }

    fn package_text(&self, package: Id<Package>) -> Result<String, VirErr> {
        let path = &self.packages[package].source;
        match std::fs::read_to_string(path) {
            Ok(source) => Ok(source),
            Err(err) => Err(err.into()),
        }
    }

    fn register_items(&mut self) {
        let packages: Vec<_> = self.packages.keys().cloned().collect();
        for package in packages {
            if let Ok(package_ast) = &self.packages[package].ast.get() {
                for node in package_ast.children() {
                    if node.is_item() {
                        self.register_item(node, package);
                    }
                }
            }
        }
    }

    fn register_item(&mut self, item_ast: Ast<'static>, package: Id<Package>) {
        let item_name = item_ast.name().unwrap();
        let qualified_item_name = format!("{package}::{item_name}");
        let item: Id<Item> = Id::new(qualified_item_name.clone());

        if self.items.is_registered(item) {
            self.errors.add(VirErr::DupItem(qualified_item_name));
        }

        let item_info = self.items.register(item);
        let kind = item_ast.item_kind().unwrap();

        item_info.name = item_name.to_string();
        item_info.kind.set(kind);
        item_info.package.set(package);
        item_info.ast.set(item_ast);
    }
}


////////////////////////////////////////////////////////////////////////////////
// Item dependencies
////////////////////////////////////////////////////////////////////////////////

impl Virdant {
    fn item_deps(&mut self, item: Id<Item>) -> Vec<Id<Item>> {
        if let Ok(item_ast) = self.items[item].ast.get() {
            let (deps, errors) =
                if item_ast.child(0).is_moddef() {
                    self.item_deps_moddef(item, item_ast.child(0))
                } else if item_ast.child(0).is_uniondef() {
                    self.item_deps_uniondef(item, item_ast.child(0))
                } else if item_ast.child(0).is_structdef() {
                    self.item_deps_structdef(item, item_ast.child(0))
                } else if item_ast.child(0).is_builtindef() {
                    (vec![], VirErrs::new())
                } else if item_ast.child(0).is_portdef() {
                    self.item_deps_moddef(item, item_ast.child(0))
                } else {
                    unreachable!()
                };

            self.errors.extend(errors);
            deps
        } else {
            vec![]
        }
    }

    fn item_deps_moddef(&self, item: Id<Item>, moddef_ast: Ast) -> (Vec<Id<Item>>, VirErrs) {
        let mut errors = VirErrs::new();
        let mut results = IndexSet::new();
        for node in moddef_ast.children() {
            if let Some(type_node) = node.typ() {
                let (deps, errs) = self.item_deps_type(type_node, item);
                errors.extend(errs);
                results.extend(deps);
            }

            if let Some(qualident) = node.of() {
                match self.resolve_qualident(&qualident, item) {
                    Ok(dep_item) => {
                        results.insert(dep_item);
                    },
                    Err(err) => {
                        errors.add(err);
                    },
                }
            }
        }
        (results.into_iter().collect(), errors)
    }

    fn item_deps_uniondef(&self, item: Id<Item>, uniondef_ast: Ast) -> (Vec<Id<Item>>, VirErrs) {
        let mut errors = VirErrs::new();
        let mut results = IndexSet::new();
        for node in uniondef_ast.children() {
            if node.is_statement() {
                let args = node.args().unwrap();
                for arg in args {
                    let arg_type = arg.typ().unwrap();
                    let (deps, errs) = self.item_deps_type(arg_type, item);
                    results.extend(deps);
                    errors.extend(errs);
                }
            }
        }
        (results.into_iter().collect(), errors)
    }

    fn item_deps_structdef(&self, item: Id<Item>, structdef_ast: Ast) -> (Vec<Id<Item>>, VirErrs) {
        let mut errors = VirErrs::new();
        let mut results = IndexSet::new();
        for node in structdef_ast.children() {
            if node.is_statement() {
                let typ = node.typ().unwrap();
                let (deps, errs) = self.item_deps_type(typ, item);
                results.extend(deps);
                errors.extend(errs);
            }
        }
        (results.into_iter().collect(), errors)
    }

    fn item_deps_type(&self, type_ast: Ast, in_item: Id<Item>) -> (Vec<Id<Item>>, VirErrs) {
        let mut errors = VirErrs::new();
        match self.resolve_qualident(type_ast.name().unwrap(), in_item) {
            Ok(item) => (vec![item], errors),
            Err(err) => {
                errors.add(err);
                (vec![], errors)
            }
        }
    }

    fn check_no_item_dep_cycles(&mut self) {
        let mut dep_graph = IndexMap::new();

        for (item, item_info) in self.items.iter() {
            let deps = item_info.deps.unwrap();
            dep_graph.insert(item.clone(), deps.to_owned());
        }

        if let Err(cycle) = detect_cycle(&dep_graph) {
            let cycle_names: Vec<String> = cycle
                .into_iter()
                .map(|item| item.to_string())
                .collect();
            self.errors.add(VirErr::ItemDepCycle(cycle_names));
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Resolution
////////////////////////////////////////////////////////////////////////////////

impl Virdant {
    fn resolve_package(&self, package_name: &str) -> Option<Id<Package>> {
        for (package, package_info) in self.packages.iter() {
            if package_name == package_info.name {
                return Some(*package);
            }
        }
        None
    }

    fn resolve_qualident(&self, qualident: &str, in_item: Id<Item>) -> Result<Id<Item>, VirErr> {
        let qi = QualIdent::new(qualident);
        let in_package = self.items[in_item].package.unwrap().clone();
        let resolved_package_name = qi.in_package(&in_package.to_string()).to_string();
        let builtin_resolved_package_name = qi.in_package("builtin").to_string();
        self.items
            .resolve(&resolved_package_name)
            .or_else(|| self.items.resolve(&builtin_resolved_package_name))
            .ok_or_else(|| VirErr::UnresolvedIdent(format!("{qualident}")))
    }
}


////////////////////////////////////////////////////////////////////////////////
// Import Checks
////////////////////////////////////////////////////////////////////////////////

impl Virdant {
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
}


////////////////////////////////////////////////////////////////////////////////
// Register types
////////////////////////////////////////////////////////////////////////////////

impl Virdant {
    fn register_structdefs(&mut self) {
        let structdefs = self.items_by_kind(ItemKind::StructDef);
        for item in structdefs {
            let structdef: Id<StructDef> = item.cast();
            let structdef_info = self.structdefs.register(structdef);
            structdef_info.item.set(item);

            let mut fields = vec![];

            let item_info = &self.items[item];

            let item_ast = if let Ok(item_ast) = item_info.ast.get() {
                item_ast
            } else {
                continue;
            };

            let structdef_ast = item_ast.child(0);
            for node in structdef_ast.children() {
                if node.is_statement() {
                    let field_name = node.name().unwrap();
                    let field_type_ast = node.typ().unwrap();

                    let field_type = match self.resolve_type(field_type_ast, item) {
                        Ok(field_type) => field_type,
                        Err(err) => {
                            self.errors.add(err);
                            continue;
                        },
                    };

                    let field: Id<Field> = Id::new(format!("{item}::{field_name}"));
                    let field_info = self.fields.register(field);
                    field_info.structdef.set(structdef);
                    field_info.name = field_name.to_string();
                    field_info.typ.set(field_type);

                    fields.push(field);
                }
            }

            let structdef_info = &mut self.structdefs[structdef];
            structdef_info.fields.set(fields);
        }
    }

    fn register_uniondefs(&mut self) {
        let uniondefs = self.items_by_kind(ItemKind::UnionDef);
        for item in uniondefs {
            let uniondef: Id<UnionDef> = item.cast();
            let uniondef_info = self.uniondefs.register(uniondef);
            uniondef_info.item.set(item);

            let mut ctors = vec![];

            let item_info = &self.items[item];
            let uniondef_ast = item_info.ast.unwrap().child(0);
            for node in uniondef_ast.children() {
                if node.is_statement() {
                    let ctor_name = node.name().unwrap();
                    let ctor_param_asts = node.args().unwrap();

                    let ctor: Id<Ctor> = Id::new(format!("{item}::{ctor_name}"));
                    let mut params: Vec<(String, Type)> = vec![];

                    for ctor_param_ast in ctor_param_asts {
                        let ctor_param_name = ctor_param_ast.name().unwrap().to_string();
                        let type_ast = ctor_param_ast.typ().unwrap();
                        let ctor_param_type = match self.resolve_type(type_ast, item) {
                            Ok(ctor_type) => ctor_type,
                            Err(err) => {
                                self.errors.add(err);
                                continue;
                            },
                        };
                        params.push((ctor_param_name, ctor_param_type));
                    }

                    let ret_typ = Type::uniondef(uniondef);
                    let ctor_sig = CtorSig::new(ctor, params, ret_typ);

                    let ctor_info = self.ctors.register(ctor);
                    ctor_info.uniondef.set(uniondef);
                    ctor_info.name = ctor_name.to_string();
                    ctor_info.sig.set(ctor_sig);

                    ctors.push(ctor);
                }
            }

            let uniondef_info = &mut self.uniondefs[uniondef];
            uniondef_info.ctors.set(ctors);
        }
    }

    fn items_by_kind(&self, kind: ItemKind) -> Vec<Id<Item>> {
        let mut items = vec![];
        for (item, item_info) in self.items.iter() {
            if *item_info.kind.unwrap() == kind {
                items.push(item.clone());
            }
        }
        items
    }

    fn resolve_type(&self, type_ast: Ast, in_item: Id<Item>) -> Result<Type, VirErr> {
        let item = self.resolve_qualident(type_ast.name().unwrap(), in_item)?;


        let mut width: Option<types::Nat> = None;

        if let Some(args) = type_ast.args() {
            assert_eq!(args.len(), 1);

            let arg_ast = args[0].child(0);
            if arg_ast.is_nat() {
                width = Some(str::parse::<u64>(arg_ast.as_str()).unwrap());
            } else {
                return Err(VirErr::KindError("Only widths are allowed here".to_string()));
            }
        }

        let item_kind = self.items[item].kind.unwrap();
        match item_kind {
            ItemKind::UnionDef => {
                if width.is_some() {
                    return Err(VirErr::KindError(format!("Union definition {item} does not take a generic")));
                }
                Ok(Type::uniondef(item.cast()))
            },
            ItemKind::StructDef => {
                if width.is_some() {
                    return Err(VirErr::KindError(format!("Struct definition {item} does not take a generic")));
                }
                Ok(Type::structdef(item.cast()))
            },
            ItemKind::BuiltinDef => {
                let word_builitindef = self.resolve_qualident("builtin::Word", in_item).unwrap();

                if item == word_builitindef && width.is_none() {
                    return Err(VirErr::KindError("Word requires a length".to_string()));
                } else if item != word_builitindef && width.is_some() {
                    return Err(VirErr::KindError(format!("Type definition {item} does not take a generic")));
                }

                Ok(Type::builtindef(item.cast(), width))
            },
            _ => unreachable!(),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// For testing
////////////////////////////////////////////////////////////////////////////////

impl Virdant {
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
    BuiltinDef,
    PortDef,
}

impl ItemKind {
    pub fn is_typedef(&self) -> bool {
        match self {
            ItemKind::ModDef => false,
            ItemKind::UnionDef => true,
            ItemKind::StructDef => true,
            ItemKind::BuiltinDef => true,
            ItemKind::PortDef => false,
        }
    }
}
