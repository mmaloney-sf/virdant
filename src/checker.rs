use std::sync::Arc;
use crate::common::*;
use crate::ast;
use crate::parse;

#[salsa::query_group(QueryGroupStorage)]
trait QueryGroup: salsa::Database {
    #[salsa::input]
    fn source(&self) -> Arc<String>;

    fn package_ast(&self) -> Result<ast::Package, VirdantError>;

    fn package_item_names(&self) -> Result<Vec<Ident>, VirdantError>;

    fn package_moddef_names(&self) -> Result<Vec<Ident>, VirdantError>;

    fn moddef_ast(&self, key: Ident) -> Result<ast::ModDef, VirdantError>;

    fn moddef_entity_names(&self, key: Ident) -> Result<Vec<Ident>, VirdantError>;

    fn moddef_component_names(&self, key: Ident) -> Result<Vec<Ident>, VirdantError>;

    fn moddef_component(&self, moddef: Ident, component: Ident) -> Result<ast::Component, VirdantError>;

//    fn moddef_entity_ast(&self, moddef: Ident, entity: Ident) -> Result<Vec<Ident>, VirdantError>;

    fn moddef_component_type(&self, moddef: Ident, component: Ident) -> Result<ast::Type, VirdantError>;

    fn moddef_component_connects(&self, moddef: Ident, entity: Ident) -> Result<Vec<ast::InlineConnect>, VirdantError>;

    fn check(&self) -> Result<(), VirdantError>;
}


fn moddef_component(db: &dyn QueryGroup, moddef: Ident, component: Ident) -> Result<ast::Component, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef)?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(c) if c.name == component => return Ok(c.clone()),
            _ => (),
        }
    }
    Err(VirdantError::Unknown)
}

fn check(db: &dyn QueryGroup) -> Result<(), VirdantError> {
    let mut errors = ErrorReport::new();
    for moddef in &db.package_moddef_names()? {
        for component in db.moddef_component_names(moddef.clone())? {
            let c = db.moddef_component(moddef.clone(), component.clone())?;
            let connects = db.moddef_component_connects(moddef.clone(), component.clone())?;
            if c.kind == ast::ComponentKind::Incoming {
                if connects.len() > 0 {
                    errors.add(VirdantError::Other(format!("connect for incoming {} in {}", component, moddef)));
                }
            } else {
                if connects.len() < 1 {
                    errors.add(VirdantError::Other(format!("no connect for {} in {}", component, moddef)));
                } else if connects.len() > 1 {
                    errors.add(VirdantError::Other(format!("multiple connects for {} in {}", component, moddef)));
                }
            }
        }
    }
    errors.check()
}

fn moddef_component_connects(db: &dyn QueryGroup, moddef: Ident, component: Ident) -> Result<Vec<ast::InlineConnect>, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef)?;
    let mut result = vec![];

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(c) if c.name == component => {
                if let Some(connect) = &c.connect {
                    result.push(connect.clone());
                }
            },
            ast::Decl::Connect(ast::Connect(target, connect_type, expr)) if target == &component.as_path() => {
                result.push(ast::InlineConnect(*connect_type, expr.clone()));
            },
            _ => (),
        }
    }
    Ok(result)
}

fn moddef_component_type(db: &dyn QueryGroup, moddef: Ident, component: Ident) -> Result<ast::Type, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef)?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(c) if c.name == component => return Ok(c.typ.clone()),
            ast::Decl::Submodule(submodule) if submodule.name == component => return Err(VirdantError::Unknown),
            _ => (),
        }
    }

    Err(VirdantError::Unknown)
}

fn package_ast(db: &dyn QueryGroup) -> Result<ast::Package, VirdantError> {
    let input = db.source();
    parse::parse_package(&input)
}

fn package_moddef_names(db: &dyn QueryGroup) -> Result<Vec<Ident>, VirdantError> {
    let package = db.package_ast()?;
    let mut result = vec![];
    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => result.push(moddef.name.clone()),
        }
    }
    Ok(result)
}

fn package_item_names(db: &dyn QueryGroup) -> Result<Vec<Ident>, VirdantError> {
    let package = db.package_ast()?;
    let mut result = vec![];
    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => result.push(moddef.name.clone()),
        }
    }
    Ok(result)
}

fn moddef_ast(db: &dyn QueryGroup, key: Ident) -> Result<ast::ModDef, VirdantError> {
    let package = db.package_ast()?;
    let mut result: Option<ast::ModDef> = None;

    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => {
                if moddef.name == key {
                    if result.is_none() {
                        result = Some(moddef.clone());
                    } else {
                        return Err(VirdantError::Other("Uh oh".into()));
                    }
                }
            }
        }
    }

    if let Some(moddef) = result {
        Ok(moddef)
    } else {
        Err(VirdantError::Unknown)
    }
}

fn moddef_component_names(db: &dyn QueryGroup, moddef: Ident) -> Result<Vec<Ident>, VirdantError> {
    let moddef = db.moddef_ast(moddef)?;
    let mut result = vec![];
    for decl in moddef.decls {
        match decl {
            ast::Decl::Component(component) => result.push(component.name.clone()),
            ast::Decl::Submodule(submodule) => (),
            ast::Decl::Connect(_connect) => (),
        }
    }
    Ok(result)
}

fn moddef_entity_names(db: &dyn QueryGroup, moddef: Ident) -> Result<Vec<Ident>, VirdantError> {
    let moddef = db.moddef_ast(moddef)?;
    let mut result = vec![];
    for decl in moddef.decls {
        match decl {
            ast::Decl::Component(component) => result.push(component.name.clone()),
            ast::Decl::Submodule(submodule) => result.push(submodule.name.clone()),
            ast::Decl::Connect(_connect) => (),
        }
    }
    Ok(result)
}

#[salsa::database(QueryGroupStorage)]
#[derive(Default)]
struct DatabaseStruct {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for DatabaseStruct {}

#[test]
fn foo() {
    let mut db = DatabaseStruct::default();
    db.set_source(Arc::new("
        public module Top {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8];
            reg r : Word[8] on clk <= in;
            out := in;
            submodule foo of Foo;
        }

        module Foo {
            wire w : Word[8] := 0;
        }
    ".to_string()));

    db.check().unwrap();
}
