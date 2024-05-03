use std::sync::Arc;
use std::collections::HashMap;
use crate::common::*;
use crate::ast;
use crate::parse;
use crate::hir;
use crate::context::Context;
use crate::types::Type;

#[salsa::query_group(QueryGroupStorage)]
pub trait QueryGroup: salsa::Database {
    #[salsa::input]
    fn source(&self) -> Arc<String>;

    fn package_ast(&self) -> Result<ast::Package, VirdantError>;

    fn package_item_names(&self) -> Result<Vec<Ident>, VirdantError>;

    fn package_moddef_names(&self) -> Result<Vec<Ident>, VirdantError>;

    fn moddef_ast(&self, moddef: Ident) -> Result<ast::ModDef, VirdantError>;

    fn moddef_hir(&self, moddef: Ident) -> VirdantResult<hir::ModDef>;

    fn moddef_entity_names(&self, moddef: Ident) -> Result<Vec<Ident>, VirdantError>;

    fn moddef_component_names(&self, moddef: Ident) -> Result<Vec<Ident>, VirdantError>;

    fn moddef_component_hir(&self, moddef: Ident, component: Ident) -> VirdantResult<hir::Component>;

    fn moddef_component(&self, moddef: Ident, component: Ident) -> Result<ast::Component, VirdantError>;

//    fn moddef_entity_ast(&self, moddef: Ident, entity: Ident) -> Result<Vec<Ident>, VirdantError>;

    fn moddef_component_type(&self, moddef: Ident, component: Ident) -> Result<ast::Type, VirdantError>;

    fn moddef_component_connects(&self, moddef: Ident, component: Ident) -> Result<Vec<ast::InlineConnect>, VirdantError>;

    fn moddef_context(&self, moddef: Ident) -> Result<Context<Path, Arc<crate::types::Type>>, VirdantError>;

    fn typecheck_component(&self, moddef: Ident, component: Ident) -> VirdantResult<hir::Expr>;

    fn check_moddef(&self, moddef: Ident) -> VirdantResult<()>;

    fn check(&self) -> Result<(), VirdantError>;

    fn package_hir(&self) -> VirdantResult<hir::Package>;
}


fn package_hir(db: &dyn QueryGroup) -> VirdantResult<hir::Package> {
    db.check()?;
    let mut moddefs = HashMap::new();

    for moddef_name in db.package_moddef_names()? {
        let moddef_hir = db.moddef_hir(moddef_name.clone())?;
        moddefs.insert(moddef_name.clone(), moddef_hir.into());
    }

    Ok(hir::Package {
        moddefs,
    })
}

fn moddef_component_hir(db: &dyn QueryGroup, moddef: Ident, component: Ident) -> VirdantResult<hir::Component> {
    let c = db.moddef_component(moddef.clone(), component.clone())?;
    let typ = crate::types::Type::from_ast(&c.typ);

    Ok(match c.kind {
        ast::ComponentKind::Incoming => hir::Component::Incoming(c.name.clone(), typ),
        ast::ComponentKind::Outgoing => {
            let expr = db.typecheck_component(moddef.clone(), component.clone())?;
            hir::Component::Outgoing(c.name.clone(), Type::from_ast(&c.typ), expr)
        },
        ast::ComponentKind::Wire => {
            let expr = db.typecheck_component(moddef.clone(), component.clone())?;
            hir::Component::Wire(c.name.clone(), Type::from_ast(&c.typ), expr)
        },
        ast::ComponentKind::Reg => {
            let ctx = db.moddef_context(moddef.clone())?;
            let clock: hir::Expr = hir::Expr::from_ast(&c.clock.unwrap());
            let clock_typed: hir::Expr = clock.typecheck(ctx, Type::from_ast(&ast::Type::Clock)).map_err(|e| VirdantError::TypeError(e))?;

            let expr = db.typecheck_component(moddef.clone(), component.clone())?;
            hir::Component::Reg(c.name.clone(), Type::from_ast(&c.typ), clock_typed, expr)
        },
    })
}

fn moddef_hir(db: &dyn QueryGroup, moddef: Ident) -> VirdantResult<hir::ModDef> {
    let mut components: Vec<hir::Component> = vec![];
    let mut submodules: Vec<hir::Submodule> = vec![];

    for decl in db.moddef_ast(moddef.clone())?.decls {
        match decl {
            ast::Decl::Submodule(m) => submodules.push(
                hir::Submodule {
                    name: m.name,
                    moddef: m.moddef,
                }
            ),
            _ => (),
        }
    }

    for component_name in db.moddef_component_names(moddef.clone())? {
        let component = db.moddef_component_hir(moddef.clone(), component_name)?;
        components.push(component);
    }

    Ok(hir::ModDef {
        name: moddef.clone(),
        components,
        submodules,
    })
}


fn moddef_context(db: &dyn QueryGroup, moddef: Ident) -> Result<Context<Path, Arc<crate::types::Type>>, VirdantError> {
    let mut ctx = Context::empty();
    for component in db.moddef_component_names(moddef.clone())? {
        let typ = crate::types::Type::from_ast(&db.moddef_component_type(moddef.clone(), component.clone())?);
        ctx = ctx.extend(component.as_path(), typ);
    }
    Ok(ctx)
}

fn typecheck_component(db: &dyn QueryGroup, moddef: Ident, component: Ident) -> VirdantResult<hir::Expr> {
    let ast::InlineConnect(_connect_type, expr) = db.moddef_component_connects(moddef.clone(), component.clone())?[0].clone();
    let ctx = db.moddef_context(moddef.clone())?;
    let typ = Type::from_ast(&db.moddef_component_type(moddef.clone(), component.clone())?);

    hir::Expr::from_ast(&expr).typecheck(ctx, typ).map_err(|err| VirdantError::Other(format!("{err:?} {moddef} {component}")))
}

/*
fn typecheck_expr(db: &dyn QueryGroup, ctx: Context<Path, Arc<crate::types::Type>>, expr: ast::Expr, typ: ast::Type) -> VirdantResult<hir::Expr> {
    let hir_expr = hir::Expr::from_ast(&expr);
    let typ = crate::types::Type::from_ast(&typ);
    hir_expr.typecheck(ctx, typ).map_err(|e| VirdantError::TypeError(e))
}
*/


fn moddef_component(db: &dyn QueryGroup, moddef: Ident, component: Ident) -> Result<ast::Component, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(c) if c.name == component => return Ok(c.clone()),
            _ => (),
        }
    }
    Err(VirdantError::Other(format!("No such moddef {}", moddef)))
}

fn check_moddef(db: &dyn QueryGroup, moddef: Ident) -> VirdantResult<()> {
    let mut errors = ErrorReport::new();
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
            } else {
                if let Err(err) = db.typecheck_component(moddef.clone(), component.clone()) {
                    errors.add(err);
                }
            }
        }
    }
    errors.check()
}

fn check(db: &dyn QueryGroup) -> Result<(), VirdantError> {
    let mut errors = ErrorReport::new();
    for moddef in &db.package_moddef_names()? {
        if let Err(err) = db.check_moddef(moddef.clone()) {
            errors.add(err);
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
            ast::Decl::Submodule(submodule) if submodule.name == component => return Err(VirdantError::Other("Submodules have no types".into())),
            _ => (),
        }
    }

    Err(VirdantError::Other("Component not found".into()))
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
pub struct DatabaseStruct {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for DatabaseStruct {}

pub fn compile(input: &str) -> VirdantResult<()> {
    let mut db = DatabaseStruct::default();
    db.set_source(Arc::new(input.to_string()));

    let package = db.package_hir()?;
    let mut stdout = std::io::stdout();
    package.mlir(&mut stdout).map_err(|_err| VirdantError::Unknown)?;
    Ok(())
}

pub fn check_module(input: &str) -> VirdantResult<hir::Package> {
    let mut db = DatabaseStruct::default();
    db.set_source(Arc::new(input.to_string()));
    Ok(db.package_hir()?)
}

#[test]
fn test_checker() {
    let mut db = DatabaseStruct::default();
    db.set_source(Arc::new("
        public module Top {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8];
            reg r : Word[8] on clk <= in;
            out := in->add(1w8);
            submodule foo of Foo;
        }

        module Foo {
            wire w : Word[8] := 0;
        }
    ".to_string()));

    db.check().unwrap();
}
