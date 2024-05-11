use crate::common::*;

pub use super::StructureQ;

use crate::hir;
use crate::types::Type;
use crate::context::Context;
use crate::ast;

#[salsa::query_group(TypecheckQStorage)]
pub trait TypecheckQ: StructureQ {
    fn moddef_context(&self, moddef: Ident) -> VirdantResult<Context<Path, Arc<Type>>>;
    fn moddef_component_type(&self, moddef: Ident, component: Ident) -> VirdantResult<ast::Type>;

    fn moddef_hir_typed(&self, moddef: Ident) -> VirdantResult<hir::ModDef>;
    fn typecheck_component(&self, moddef: Ident, component: Ident) -> VirdantResult<hir::Expr>;
    fn moddef_component_hir_typed(&self, moddef: Ident, component: Ident) -> VirdantResult<hir::Component>;

    fn moddef_submodule_connects_typed(&self, moddef: Ident, submodule: Ident) -> VirdantResult<Vec<hir::Connect>>;
    fn moddef_nonlocal_port_type(&self, moddef: Ident, reference: Path) -> VirdantResult<Arc<Type>>;
}

fn moddef_component_hir_typed(db: &dyn TypecheckQ, moddef: Ident, component: Ident) -> VirdantResult<hir::Component> {
    let c = db.moddef_component_ast(moddef.clone(), component.clone())?;
    let typ = Type::from_ast(&c.typ);

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
            let clock = c.clock.ok_or_else(|| VirdantError::Other(format!("No \"on\" clause for reg")))?;

            let expr = db.typecheck_component(moddef.clone(), component.clone())?;
            hir::Component::Reg(c.name.clone(), Type::from_ast(&c.typ), clock, expr)
        },
    })
}

fn moddef_hir_typed(db: &dyn TypecheckQ, moddef: Ident) -> VirdantResult<hir::ModDef> {
    let mut components: Vec<hir::Component> = vec![];
    let mut submodules: Vec<hir::Submodule> = vec![];
    let mut connects: Vec<hir::Connect> = vec![];

    for decl in db.moddef_ast(moddef.clone())?.decls {
        match decl {
            ast::Decl::Submodule(m) => submodules.push(
                hir::Submodule {
                    name: m.name,
                    moddef: m.moddef,
                }
            ),
            ast::Decl::Connect(ast::Connect(target, connect_type, expr_ast)) => {
                if target.is_nonlocal() {
                    let ctx = db.moddef_context(moddef.clone())?;
                    let typ = db.moddef_nonlocal_port_type(moddef.clone(), target.clone())?;
                    let expr = hir::Expr::from_ast(&expr_ast);
                    let expr_typed = expr.typecheck(ctx, typ)?;
                    connects.push(hir::Connect(target.clone(), connect_type, expr_typed));
                } else {
                    // TODO
                }
            },
            _ => (),
        }
    }

    for component_name in db.moddef_component_names(moddef.clone())? {
        let component = db.moddef_component_hir_typed(moddef.clone(), component_name)?;
        components.push(component);
    }

    Ok(hir::ModDef {
        name: moddef.clone(),
        components,
        submodules,
        connects,
    })
}

fn moddef_context(db: &dyn TypecheckQ, moddef: Ident) -> Result<Context<Path, Arc<Type>>, VirdantError> {
    let mut ctx = Context::empty();
    for component in db.moddef_component_names(moddef.clone())? {
        let typ = Type::from_ast(&db.moddef_component_type(moddef.clone(), component.clone())?);
        ctx = ctx.extend(component.as_path(), typ);
    }

    for submodule in db.moddef_submodules(moddef.clone())? {
        let submodule_moddef = db.moddef_hir(submodule.moddef.clone())?;
        for component in &submodule_moddef.components {
            if let hir::Component::Incoming(name, typ) = component {
                let path = submodule.name.as_path().join(&name.as_path());
                ctx = ctx.extend(path, typ.clone());
            } else if let hir::Component::Outgoing(name, typ, _connect) = component {
                let path = submodule.name.as_path().join(&name.as_path());
                ctx = ctx.extend(path, typ.clone());
            }
        }
    }
    Ok(ctx)
}

fn typecheck_component(db: &dyn TypecheckQ, moddef: Ident, component: Ident) -> VirdantResult<hir::Expr> {
    let ast::InlineConnect(_connect_type, expr) = db.moddef_component_connects(moddef.clone(), component.clone())?[0].clone();
    let ctx = db.moddef_context(moddef.clone())?;
    let typ = Type::from_ast(&db.moddef_component_type(moddef.clone(), component.clone())?);

    hir::Expr::from_ast(&expr).typecheck(ctx, typ).map_err(|err| VirdantError::Other(format!("{err:?} {moddef} {component}")))
}

fn moddef_component_type(db: &dyn TypecheckQ, moddef: Ident, component: Ident) -> Result<ast::Type, VirdantError> {
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

fn moddef_submodule_connects_typed(db: &dyn TypecheckQ, moddef: Ident, submodule: Ident) -> VirdantResult<Vec<hir::Connect>> {
    let mut result = vec![];
    let submodule_moddef = db.moddef_submodule_moddef(moddef.clone(), submodule.clone())?;
    for ast::Connect(target, connect_type, expr) in db.moddef_submodule_connects(moddef.clone(), submodule)? {
        let component: Ident = target.parts()[1].into();
        let ctx = db.moddef_context(moddef.clone())?;
        let typ = Type::from_ast(&db.moddef_component_type(submodule_moddef.clone(), component.clone())?);
        let expr_typed = hir::Expr::from_ast(&expr).typecheck(ctx, typ).map_err(|err| VirdantError::Other(format!("{err:?} {moddef} {component}")))?;

        result.push(hir::Connect(target.clone(), connect_type, expr_typed));
    }
    Ok(result)
}

fn moddef_nonlocal_port_type(db: &dyn TypecheckQ, moddef: Ident, reference: Path) -> VirdantResult<Arc<Type>> {
    let reference_parts = reference.parts();
    let submodule_name: Ident= reference_parts[0].into();
    let submodule_port: Ident = reference_parts[1].into();

    let submodule_moddef = db.moddef_submodule_moddef(moddef.clone(), submodule_name.clone())?;

    let type_ast = db.moddef_component_type(submodule_moddef, submodule_port)?;
    Ok(Type::from_ast(&type_ast))
}
