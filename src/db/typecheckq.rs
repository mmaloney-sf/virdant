use crate::common::*;

pub use super::StructureQ;

use crate::types::Type;
use crate::context::Context;
use crate::ast;

#[salsa::query_group(TypecheckQStorage)]
pub trait TypecheckQ: StructureQ {
    fn resolve_type(&self, typ: Arc<ast::Type>) -> VirdantResult<Type>;

    fn moddef_context(&self, moddef: Ident) -> VirdantResult<Context<Path, Type>>;
    fn moddef_component_type(&self, moddef: Ident, component: Ident) -> VirdantResult<Type>;

    fn moddef_typecheck(&self, moddef: Ident) -> VirdantResult<()>;
    fn typecheck(&self) -> VirdantResult<()>;
}

fn typecheck(db: &dyn TypecheckQ) -> VirdantResult<()> {
    let mut errors = ErrorReport::new();
    for moddef in db.package_moddef_names()? {
        if let Err(err) = db.moddef_typecheck(moddef) {
            errors.add(err);
        }
    }
    errors.check()
}

fn moddef_typecheck(db: &dyn TypecheckQ, moddef: Ident) -> VirdantResult<()> {
    db.moddef_context(moddef.clone())?;
    Ok(())
}

fn moddef_context(db: &dyn TypecheckQ, moddef: Ident) -> Result<Context<Path, Type>, VirdantError> {
    let mut ctx = Context::empty();
    for component in db.moddef_component_names(moddef.clone())? {
        let typ = db.moddef_component_type(moddef.clone(), component.clone())?;
        ctx = ctx.extend(component.as_path(), typ);
    }

    for submodule in db.moddef_submodules(moddef.clone())? {
        for component in &db.moddef_component_names(submodule.moddef.clone())? {
            let component_ast = db.moddef_component_ast(submodule.moddef.clone(), component.clone())?;
            if let ast::SimpleComponentKind::Incoming = component_ast.kind {
                let path = submodule.name.as_path().join(&component_ast.name.as_path());
                let typ = db.moddef_component_type(submodule.moddef.clone(), component.clone())?;
                ctx = ctx.extend(path, typ);
            } else if let ast::SimpleComponentKind::Outgoing = component_ast.kind {
                let path = submodule.name.as_path().join(&component_ast.name.as_path());
                let typ = db.moddef_component_type(submodule.moddef.clone(), component.clone())?;
                ctx = ctx.extend(path, typ);
            }
        }
    }

    Ok(ctx)
}

fn moddef_component_type(db: &dyn TypecheckQ, moddef: Ident, component: Ident) -> Result<Type, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(c) if c.name == component => {
                let typ = db.resolve_type(c.typ.clone())?;
                return Ok(typ);
            },
            ast::Decl::Submodule(submodule) if submodule.name == component => return Err(VirdantError::Other("Submodules have no types".into())),
            _ => (),
        }
    }

    Err(VirdantError::Other(format!("Component not found: `{component}` in `{moddef}`")))
}

fn resolve_type(db: &dyn TypecheckQ, typ: Arc<ast::Type>) -> VirdantResult<Type> {
    let typ = match &*typ {
        ast::Type::Clock => Type::Clock.into(),
        ast::Type::Word(width) => Type::Word(*width).into(),
        ast::Type::Vec(inner, len) => Type::Vec(Arc::new(db.resolve_type(inner.clone())?), *len).into(),
        ast::Type::TypeRef(name) => Type::TypeRef(name.clone()).into(),
    };
    Ok(typ)
}
