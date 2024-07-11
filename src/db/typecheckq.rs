use std::collections::HashSet;
use crate::common::*;
use crate::value::Value;

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

    fn bitwidth(&self, typ: Type) -> VirdantResult<Width>;

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
            let actual_typ = db.moddef_reference_type(moddef, path.clone())?;
            if typ != actual_typ {
                Err(VirdantError::Other(format!("Wrong types: {path} is {typ} vs {actual_typ}")))
            } else {
                Ok(TypedExpr::Reference(typ, path.clone()).into())
            }
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
                (typ, _width) => Err(VirdantError::Other(format!("Could not typecheck {lit:?} as {typ}"))),
            }
        },
        ast::Expr::Vec(_) => todo!(),
        ast::Expr::Struct(_, _) => todo!(),
        ast::Expr::MethodCall(subject, method, args) => {
            let typed_subject = db.expr_typeinfer(moddef.clone(), subject.clone())?;
            let MethodSig(arg_types, ret_type) = db.method_sig(typed_subject.typ(), method.clone())?;

            if ret_type != typ {
                return Err(VirdantError::Other(format!("Wrong return type")));
            }

            if args.len() != arg_types.len() {
                return Err(VirdantError::Other(format!("Wrong argument list length")));
            }

            let mut typed_args = vec![];
            for (arg, arg_type) in args.iter().zip(arg_types) {
                let typed_arg = db.expr_typecheck(moddef.clone(), arg.clone(), arg_type)?;
                typed_args.push(typed_arg);
            }

            Ok(TypedExpr::MethodCall(typ, typed_subject, method.clone(), typed_args).into())
        },
        ast::Expr::As(subject, expected_typ) => {
            let expected_type_resolved = db.resolve_type(expected_typ.clone())?;
            let typed_subject = db.expr_typecheck(moddef.clone(), subject.clone(), expected_type_resolved.clone())?;
            if expected_type_resolved != typ {
                return Err(VirdantError::Unknown);
            } else {
                Ok(TypedExpr::As(expected_type_resolved, typed_subject.clone(), expected_typ.clone()).into())
            }
        },
        ast::Expr::Idx(_subject, _i) => {
            let typed_expr = db.expr_typeinfer(moddef, expr)?;
            if typed_expr.typ() != typ {
                Err(VirdantError::Unknown)
            } else {
                Ok(typed_expr)
            }
        },
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
        ast::Expr::MethodCall(subject, method, args) => {
            let typed_subject = db.expr_typeinfer(moddef.clone(), subject.clone())?;
            let MethodSig(arg_types, ret_typ) = db.method_sig(typed_subject.typ(), method.clone())?;

            if args.len() != arg_types.len() {
                return Err(VirdantError::Unknown);
            }

            let mut typed_args = vec![];
            for (arg, arg_type) in args.iter().zip(arg_types) {
                let typed_arg = db.expr_typecheck(moddef.clone(), arg.clone(), arg_type)?;
                typed_args.push(typed_arg);
            }

            Ok(TypedExpr::MethodCall(ret_typ, typed_subject, method.clone(), typed_args).into())
        },
        ast::Expr::As(_, _) => todo!(),
        ast::Expr::Idx(subject, i) => {
            eprintln!("TODO: Check i fits in the size of the subject");
            let typed_subject = db.expr_typeinfer(moddef.clone(), subject.clone())?;
            Ok(TypedExpr::Idx(Type::Word(1), typed_subject, *i).into())
        },
        ast::Expr::IdxRange(_, _, _) => todo!(),
        ast::Expr::Cat(_) => todo!(),
        ast::Expr::If(_, _, _) => Err(VirdantError::Other("Can't infer".to_string())),
    }
}

fn method_sig(_db: &dyn TypecheckQ, typ: Type, method: Ident) -> VirdantResult<MethodSig> {
    match typ {
        Type::Word(_n) => {
            if method == "add".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "sub".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "and".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "or".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "lt".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "lte".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "gt".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "gte".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "eq".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "neq".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "not".into() {
                Ok(MethodSig(vec![], typ.clone()))
            } else {
                Err(VirdantError::Other(format!("No such method {method} for type {typ}")))
            }
        },
        _ => Err(VirdantError::Other(format!("No such method {method} for type {typ}"))),
    }
}

fn bitwidth(_db: &dyn TypecheckQ, typ: Type) -> VirdantResult<Width> {
    match typ {
        Type::Unknown => todo!(),
        Type::Clock => Ok(1),
        Type::Bool => Ok(1),
        Type::Word(n) => Ok(n.into()),
        Type::Vec(_, _) => todo!(),
        Type::TypeRef(_) => todo!(),
        Type::Other(_) => todo!(),
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

fn resolve_type(db: &dyn TypecheckQ, typ: Arc<ast::Type>) -> VirdantResult<Type> {
    let typ = match &*typ {
        ast::Type::Clock => Type::Clock.into(),
        ast::Type::Word(width) => Type::Word(*width).into(),
        ast::Type::Vec(inner, len) => Type::Vec(Arc::new(db.resolve_type(inner.clone())?), *len).into(),
        ast::Type::TypeRef(name) => Type::TypeRef(name.clone()).into(),
    };
    Ok(typ)
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
    As(Type, Arc<TypedExpr>, Arc<ast::Type>),
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

    pub fn references(&self) -> HashSet<Path> {
        match self {
            TypedExpr::Reference(_typ, path) => vec![path.clone()].into_iter().collect(),
            TypedExpr::Word(_typ, _) => HashSet::new(),
            TypedExpr::Vec(_typ, _) => HashSet::new(),
            TypedExpr::Struct(_typ, _, _) => HashSet::new(),
            TypedExpr::MethodCall(_typ, _, _, _) => HashSet::new(),
            TypedExpr::As(_typ, _, _) => HashSet::new(),
            TypedExpr::Idx(_typ, _, _) => HashSet::new(),
            TypedExpr::IdxRange(_typ, _, _, _) => HashSet::new(),
            TypedExpr::Cat(_typ, _) => HashSet::new(),
            TypedExpr::If(_typ, _, _, _) => HashSet::new(),
        }
    }

    pub fn eval(&self, ctx: Context<Path, Value>) -> Value {
        match self {
            TypedExpr::Reference(_typ, path) => ctx.lookup(path).unwrap(),
            TypedExpr::Word(typ, lit) => {
                if let Type::Word(n) = typ {
                    Value::Word(*n, lit.value)
                } else {
                    panic!()
                }
            },
            TypedExpr::Vec(_typ, _) => todo!(),
            TypedExpr::Struct(_typ, _, _) => todo!(),
            TypedExpr::MethodCall(_typ, _, _, _) => todo!(),
            TypedExpr::As(_typ, _, _) => todo!(),
            TypedExpr::Idx(_typ, _, _) => todo!(),
            TypedExpr::IdxRange(_typ, _, _, _) => todo!(),
            TypedExpr::Cat(_typ, _) => todo!(),
            TypedExpr::If(_typ, _, _, _) => todo!(),
        }
    }
}
