use crate::common::*;

pub use super::StructureQ;

use crate::types::Type;
use crate::context::Context;
use crate::ast::{self, WordLit};

#[salsa::query_group(TypecheckQStorage)]
pub trait TypecheckQ: StructureQ {
    fn resolve_type(&self, typ: Arc<ast::Type>) -> VirdantResult<Type>;

    fn moddef_full_context(&self, moddef: Ident) -> VirdantResult<Context<Path, Type>>;
    fn moddef_component_type(&self, moddef: Ident, component: Ident) -> VirdantResult<Type>;
    fn moddef_reference_type(&self, moddef: Ident, path: Path) -> VirdantResult<Type>;
    fn moddef_target_type(&self, moddef: Ident, target: Path) -> VirdantResult<Type>;

    fn expr_typecheck(&self, moddef: Ident, expr: Arc<ast::Expr>, typ: Type) -> VirdantResult<Arc<TypedExpr>>;
    fn expr_typeinfer(&self, moddef: Ident, expr: Arc<ast::Expr>) -> VirdantResult<Arc<TypedExpr>>;

    fn method_sig(&self, typ: Type, method: Ident) -> VirdantResult<MethodSig>;

    fn moddef_typecheck_wire(&self, moddef: Ident, target: Path) -> VirdantResult<Arc<TypedExpr>>;
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

fn expr_typecheck(db: &dyn TypecheckQ, moddef: Ident, expr: Arc<ast::Expr>, typ: Type) -> VirdantResult<Arc<TypedExpr>> {
    match expr.as_ref() {
        ast::Expr::Reference(path) => {
            let typ = db.moddef_reference_type(moddef, path.clone())?;
            Ok(TypedExpr::Reference(typ, path.clone()).into())
        },
        ast::Expr::Word(lit) => {
            match (typ.clone(), lit.width) {
                (Type::Word(n), Some(m)) if n == m => Ok(TypedExpr::Word(typ, lit.clone()).into()),
                (Type::Word(n), Some(m)) => Err(VirdantError::Other(format!("Does not match: {n} and {m}"))),
                (Type::Word(n), None) => {
                    if lit.value < pow(2, n) {
                        Ok(TypedExpr::Word(typ, lit.clone()).into())
                    } else {
                        Err(VirdantError::Other("Doesn't fit".to_string()))
                    }
                },
                (_, _) => Err(VirdantError::Unknown),
            }
        },
        ast::Expr::Vec(_) => todo!(),
        ast::Expr::Struct(_, _) => todo!(),
        ast::Expr::MethodCall(subject, method, args) => {
            let typed_subject = db.expr_typeinfer(moddef.clone(), subject.clone())?;
            let MethodSig(arg_types, ret_type) = db.method_sig(typed_subject.typ(), method.clone())?;

            if ret_type != typ {
                return Err(VirdantError::Unknown);
            }

            if args.len() != arg_types.len() {
                return Err(VirdantError::Unknown);
            }

            let mut typed_args = vec![];
            for (arg, arg_type) in args.iter().zip(arg_types) {
                let typed_arg = db.expr_typecheck(moddef.clone(), arg.clone(), arg_type)?;
                typed_args.push(typed_arg);
            }

            Ok(TypedExpr::MethodCall(typ, typed_subject, method.clone(), typed_args).into())
        },
        ast::Expr::As(_, _) => todo!(),
        ast::Expr::Idx(_, _) => todo!(),
        ast::Expr::IdxRange(_, _, _) => todo!(),
        ast::Expr::Cat(_) => todo!(),
        ast::Expr::If(c, a, b) => {
            let typed_c = db.expr_typecheck(moddef.clone(), c.clone(), Type::Word(1))?;
            let typed_a = db.expr_typecheck(moddef.clone(), a.clone(), typ.clone())?;
            let typed_b = db.expr_typecheck(moddef.clone(), b.clone(), typ.clone())?;
            Ok(TypedExpr::If(typ, typed_c, typed_a, typed_b).into())
        },
    }
}

fn expr_typeinfer(db: &dyn TypecheckQ, moddef: Ident, expr: Arc<ast::Expr>) -> VirdantResult<Arc<TypedExpr>> {
    match expr.as_ref() {
        ast::Expr::Reference(path) => {
            let typ = db.moddef_reference_type(moddef, path.clone())?;
            Ok(TypedExpr::Reference(typ, path.clone()).into())
        },
        ast::Expr::Word(lit) => {
            if let Some(n) = lit.width {
                Ok(TypedExpr::Word(Type::Word(n), lit.clone()).into())
            } else {
                Err(VirdantError::Unknown)
            }
        },
        ast::Expr::Vec(_) => todo!(),
        ast::Expr::Struct(_, _) => todo!(),
        ast::Expr::MethodCall(_, _, _) => todo!(),
        ast::Expr::As(_, _) => todo!(),
        ast::Expr::Idx(_, _) => todo!(),
        ast::Expr::IdxRange(_, _, _) => todo!(),
        ast::Expr::Cat(_) => todo!(),
        ast::Expr::If(_, _, _) => todo!(),
    }
}

fn method_sig(db: &dyn TypecheckQ, typ: Type, method: Ident) -> VirdantResult<MethodSig> {
    match typ {
        Type::Word(n) => {
            if method == "add".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else {
                Err(VirdantError::Other(format!("No such method {method} for type {typ}")))
            }
        },
        _ => Err(VirdantError::Other(format!("No such method {method} for type {typ}"))),
    }
}

fn moddef_typecheck_wire(db: &dyn TypecheckQ, moddef: Ident, target: Path) -> VirdantResult<Arc<TypedExpr>> {
    let ast::Wire(target, _wire_type, expr) = db.moddef_wire(moddef.clone(), target)?;
    let typ = db.moddef_target_type(moddef.clone(), target)?;
    Ok(db.expr_typecheck(moddef, expr, typ)?)
}

fn moddef_typecheck(db: &dyn TypecheckQ, moddef: Ident) -> VirdantResult<()> {
    let mut errors = ErrorReport::new();
    let targets = db.moddef_wire_targets(moddef.clone())?;

    for target in &targets {
        eprintln!("typechecking {target}");
        if let Err(e) = db.moddef_typecheck_wire(moddef.clone(), target.clone()) {
            errors.add(e);
        }
    }

    errors.check()
}

fn moddef_full_context(db: &dyn TypecheckQ, moddef: Ident) -> Result<Context<Path, Type>, VirdantError> {
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

fn moddef_reference_type(db: &dyn TypecheckQ, moddef: Ident, path: Path) -> VirdantResult<Type> {
    db.moddef_target_type(moddef, path)
}

fn moddef_target_type(db: &dyn TypecheckQ, moddef: Ident, target: Path) -> VirdantResult<Type> {
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(c) if c.name.as_path() == target => {
                let typ = db.resolve_type(c.typ.clone())?;
                return Ok(typ);
            },
            ast::Decl::Submodule(submodule) if submodule.name.as_path() == target.parent() => {
                return db.moddef_component_type(submodule.moddef.clone(), target.parts()[1].into());
            },
            _ => (),
        }
    }

    Err(VirdantError::Other(format!("Component not found: `{target}` in `{moddef}`")))
}

fn typecheck_wire(db: &dyn TypecheckQ, moddef: Ident, target: Path) -> VirdantResult<()> {
    let ast::Wire(target, _wire_type, expr) = db.moddef_wire(moddef.clone(), target)?;
    let expected_type = db.moddef_target_type(moddef.clone(), target.clone())?;
    let ctx = db.moddef_full_context(moddef.clone())?;
    dbg!(&ctx);
    match &*expr {
        ast::Expr::Reference(path) => {
            if let Some(actual_type) = ctx.lookup(path) {
                if actual_type == expected_type {
                    return Ok(());
                } else {
                    todo!()
                }
            } else {
                todo!()
            }
        },
//        ast::Expr::MethodCall(subject, , )
        _ => todo!(),
    }
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

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TypeTree {
    typ: Type,
    children: Vec<Arc<TypeTree>>,
}

impl TypeTree {
    pub fn new(typ: Type) -> TypeTree {
        TypeTree {
            typ,
            children: vec![],
        }
    }

    pub fn add(&mut self, type_tree: TypeTree) {
        self.children.push(Arc::new(type_tree));
    }

    pub fn typ(&self) -> Type {
        self.typ.clone()
    }
}

fn pow(n: u64, k: u64) -> u64 {
    let mut p = 1;
    for _ in 0..k {
        p *= n
    }
    p
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct MethodSig(Vec<Type>, Type);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypedExpr {
    Reference(Type, Path),
    Word(Type, WordLit),
    Vec(Type, Vec<Arc<TypedExpr>>),
    Struct(Type, Ident, Vec<(Field, Arc<TypedExpr>)>),
    MethodCall(Type, Arc<TypedExpr>, Ident, Vec<Arc<TypedExpr>>),
    As(Type, Arc<TypedExpr>, Arc<Type>),
    Idx(Type, Arc<TypedExpr>, StaticIndex),
    IdxRange(Type, Arc<TypedExpr>, StaticIndex, StaticIndex),
    Cat(Type, Vec<Arc<TypedExpr>>),
    If(Type, Arc<TypedExpr>, Arc<TypedExpr>, Arc<TypedExpr>),
}

impl TypedExpr {
    pub fn typ(&self) -> Type {
        match self {
            TypedExpr::Reference(typ, _) => typ.clone(),
            TypedExpr::Word(typ, _) => typ.clone(),
            TypedExpr::Vec(typ, _) => typ.clone(),
            TypedExpr::Struct(typ, _, _) => typ.clone(),
            TypedExpr::MethodCall(typ, _, _, _) => typ.clone(),
            TypedExpr::As(typ, _, _) => typ.clone(),
            TypedExpr::Idx(typ, _, _) => typ.clone(),
            TypedExpr::IdxRange(typ, _, _, _) => typ.clone(),
            TypedExpr::Cat(typ, _) => typ.clone(),
            TypedExpr::If(typ, _, _, _) => typ.clone(),
        }
    }
}
