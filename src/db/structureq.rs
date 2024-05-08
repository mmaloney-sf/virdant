use crate::common::*;
use crate::hir;
use crate::ast;
use crate::types::Type;
pub use super::AstQ;

#[salsa::query_group(StructureQStorage)]
pub trait StructureQ: AstQ {
    fn package_item_names(&self) -> Result<Vec<Ident>, VirdantError>;
    fn package_moddef_names(&self) -> Result<Vec<Ident>, VirdantError>;
    fn moddef_hir(&self, moddef: Ident) -> VirdantResult<hir::ModDef>;
    fn moddef_component_names(&self, moddef: Ident) -> Result<Vec<Ident>, VirdantError>;
    fn moddef_submodules(&self, moddef: Ident) -> Result<Vec<hir::Submodule>, VirdantError>;
    fn moddef_component_hir(&self, moddef: Ident, component: Ident) -> VirdantResult<hir::Component>;
    fn moddef_component(&self, moddef: Ident, component: Ident) -> Result<ast::Component, VirdantError>;
    fn moddef_component_connects(&self, moddef: Ident, component: Ident) -> Result<Vec<ast::InlineConnect>, VirdantError>;
}

fn moddef_component_hir(db: &dyn StructureQ, moddef: Ident, component: Ident) -> VirdantResult<hir::Component> {
    let c = db.moddef_component(moddef.clone(), component.clone())?;
    let typ = Type::from_ast(&c.typ);

    Ok(match c.kind {
        ast::ComponentKind::Incoming => hir::Component::Incoming(c.name.clone(), typ),
        ast::ComponentKind::Outgoing => {
            let ast::InlineConnect(_connect_type, expr) = db.moddef_component_connects(moddef.clone(), component.clone())?[0].clone();
            let expr = hir::Expr::from_ast(&expr);
            hir::Component::Outgoing(c.name.clone(), Type::from_ast(&c.typ), expr)
        },
        ast::ComponentKind::Wire => {
            let ast::InlineConnect(_connect_type, expr) = db.moddef_component_connects(moddef.clone(), component.clone())?[0].clone();
            let expr = hir::Expr::from_ast(&expr);
            hir::Component::Wire(c.name.clone(), Type::from_ast(&c.typ), expr)
        },
        ast::ComponentKind::Reg => {
            let ast::InlineConnect(_connect_type, expr) = db.moddef_component_connects(moddef.clone(), component.clone())?[0].clone();
            let expr = hir::Expr::from_ast(&expr);
            let clock: hir::Expr = hir::Expr::from_ast(&c.clock.unwrap());
            hir::Component::Reg(c.name.clone(), Type::from_ast(&c.typ), clock, expr)
        },
    })
}

fn moddef_hir(db: &dyn StructureQ, moddef: Ident) -> VirdantResult<hir::ModDef> {
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

fn package_item_names(db: &dyn StructureQ) -> Result<Vec<Ident>, VirdantError> {
    let package = db.package_ast()?;
    let mut result = vec![];
    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => result.push(moddef.name.clone()),
        }
    }
    Ok(result)
}

fn moddef_component_names(db: &dyn StructureQ, moddef: Ident) -> Result<Vec<Ident>, VirdantError> {
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

fn moddef_submodules(db: &dyn StructureQ, moddef: Ident) -> Result<Vec<hir::Submodule>, VirdantError> {
    let moddef_hir = db.moddef_hir(moddef.clone())?;
    Ok(moddef_hir.submodules.iter().cloned().collect())
}

fn package_moddef_names(db: &dyn StructureQ) -> Result<Vec<Ident>, VirdantError> {
    let package = db.package_ast()?;
    let mut result = vec![];
    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => result.push(moddef.name.clone()),
        }
    }
    Ok(result)
}

fn moddef_component(db: &dyn StructureQ, moddef: Ident, component: Ident) -> Result<ast::Component, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(c) if c.name == component => return Ok(c.clone()),
            _ => (),
        }
    }
    Err(VirdantError::Other(format!("No such moddef {}", moddef)))
}

fn moddef_component_connects(db: &dyn StructureQ, moddef: Ident, component: Ident) -> Result<Vec<ast::InlineConnect>, VirdantError> {
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
