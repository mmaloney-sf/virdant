use crate::common::*;
use crate::context::*;
use crate::ast;
use crate::virdant_error;
use super::*;

#[salsa::query_group(TypecheckQStorage)]
pub trait TypecheckQ: type_resolution::TypeResolutionQ {
    fn typecheck_expr(&self, moddef: ModDefId, expr: Arc<ast::Expr>, typ: Type, ctx: Context<Ident, Type>) -> VirdantResult<Arc<TypedExpr>>;
    fn typeinfer_expr(&self, moddef: ModDefId, expr: Arc<ast::Expr>, ctx: Context<Ident, Type>) -> VirdantResult<Arc<TypedExpr>>;

    fn moddef_reference_type(&self, moddef: ModDefId, target: Path) -> VirdantResult<Type>;

    fn typecheck_moddef(&self, moddef: ModDefId) -> VirdantResult<()>;
    fn typecheck(&self, moddef: ModDefId) -> VirdantResult<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypedExpr {
    Reference(Type, Referent),
    Word(Type, ast::WordLit),
    Vec(Type, Vec<Arc<TypedExpr>>),
    Struct(Type, Ident, Vec<(Ident, Arc<TypedExpr>)>),
    MethodCall(Type, Arc<TypedExpr>, Ident, Vec<Arc<TypedExpr>>),
    Ctor(Type, Ident, Vec<Arc<TypedExpr>>),
    As(Type, Arc<TypedExpr>, Arc<ast::Type>),
    Idx(Type, Arc<TypedExpr>, StaticIndex),
    IdxRange(Type, Arc<TypedExpr>, StaticIndex, StaticIndex),
    Cat(Type, Vec<Arc<TypedExpr>>),
    If(Type, Arc<TypedExpr>, Arc<TypedExpr>, Arc<TypedExpr>),
    Let(Type, Ident, Option<Arc<ast::Type>>, Arc<TypedExpr>, Arc<TypedExpr>),
    Match(Type, Arc<TypedExpr>, Option<Arc<Type>>, Vec<TypedMatchArm>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Referent {
    Local(Ident),
    LocalComponent(ComponentId),
    NonLocalComponent(ElementId, ComponentId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypedPat {
    At(Type, Ident, Vec<TypedPat>),
    Bind(Type, Ident),
    Otherwise(Type),
}

impl TypedPat {
    fn from(pat: &ast::Pat, typ: Type, db: &dyn TypecheckQ) -> VirdantResult<TypedPat> {
        match pat {
            ast::Pat::At(ctor, subpats) => {
                let CtorSig(arg_typs, _typ) = db.ctor_sig(typ.clone(), ctor.clone())?;

                if arg_typs.len() != subpats.len() {
                    return Err(virdant_error!("Number of arguments wrong in pattern: {arg_typs:?}"));
                }

                let mut typed_args: Vec<TypedPat> = vec![];
                for (subpat, arg_typ) in subpats.iter().zip(arg_typs) {
                    let typed_arg = TypedPat::from(subpat, arg_typ, db)?;
                    typed_args.push(typed_arg);
                }

                Ok(TypedPat::At(typ, ctor.clone(), typed_args))
            },
            ast::Pat::Bind(x) => Ok(TypedPat::Bind(typ, x.clone())),
            ast::Pat::Otherwise => Ok(TypedPat::Otherwise(typ)),
        }
    }

    pub fn typ(&self) -> Type {
        match self {
            TypedPat::At(typ, _, _) => typ.clone(),
            TypedPat::Bind(typ, _) => typ.clone(),
            TypedPat::Otherwise(typ) => typ.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedMatchArm(pub TypedPat, pub Arc<TypedExpr>);

impl TypedExpr {
    pub fn typ(&self) -> Type {
        match self {
            TypedExpr::Reference(typ, _) => typ.clone(),
            TypedExpr::Word(typ, _) => typ.clone(),
            TypedExpr::Vec(typ, _) => typ.clone(),
            TypedExpr::Struct(typ, _, _) => typ.clone(),
            TypedExpr::MethodCall(typ, _, _, _) => typ.clone(),
            TypedExpr::Ctor(typ, _, _) => typ.clone(),
            TypedExpr::As(typ, _, _) => typ.clone(),
            TypedExpr::Idx(typ, _, _) => typ.clone(),
            TypedExpr::IdxRange(typ, _, _, _) => typ.clone(),
            TypedExpr::Cat(typ, _) => typ.clone(),
            TypedExpr::If(typ, _, _, _) => typ.clone(),
            TypedExpr::Let(typ, _x, _ascription, _e, _b) => typ.clone(),
            TypedExpr::Match(typ, _subject, _ascription, _arms) => typ.clone(),
        }
    }
}

fn pow(n: u64, k: u64) -> u64 {
    let mut p = 1;
    for _ in 0..k {
        p *= n
    }
    p
}

fn typecheck_expr(
    db: &dyn TypecheckQ,
    moddef_id: ModDefId,
    expr: Arc<ast::Expr>,
    typ: Type,
    ctx: Context<Ident, Type>,
) -> VirdantResult<Arc<TypedExpr>> {
    match expr.as_ref() {
        ast::Expr::Reference(path) => {
            let expr = db.typeinfer_expr(moddef_id, expr.clone(), ctx)?;
            let actual_typ = expr.typ();
            if typ != actual_typ {
                Err(VirdantError::Other(format!("Wrong types: {path} is {typ} vs {actual_typ}")))
            } else {
                Ok(expr)
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
            let typed_subject = db.typeinfer_expr(moddef_id.clone(), subject.clone(), ctx.clone())?;
            let MethodSig(arg_types, ret_type) = db.method_sig(typed_subject.typ(), method.clone())?;

            if ret_type != typ {
                return Err(VirdantError::Other(format!("Wrong return type")));
            }

            if args.len() != arg_types.len() {
                return Err(VirdantError::Other(format!("Wrong argument list length")));
            }

            let mut typed_args = vec![];
            for (arg, arg_type) in args.iter().zip(arg_types) {
                let typed_arg = db.typecheck_expr(moddef_id.clone(), arg.clone(), arg_type, ctx.clone())?;
                typed_args.push(typed_arg);
            }

            Ok(TypedExpr::MethodCall(typ, typed_subject, method.clone(), typed_args).into())
        },
        ast::Expr::Ctor(ctor, args) => {
            let CtorSig(arg_types, _ctor_typ) = db.ctor_sig(typ.clone(), ctor.clone())?;
            if args.len() != arg_types.len() {
                return Err(VirdantError::Other("Wrong number of args".into()));
            }
            let mut typed_args = vec![];
            for (arg, arg_typ) in args.iter().zip(arg_types) {
                let typed_arg = db.typecheck_expr(moddef_id.clone(), arg.clone(), arg_typ.clone(), ctx.clone())?;
                typed_args.push(typed_arg);
            }
            Ok(TypedExpr::Ctor(typ, ctor.clone(), typed_args).into())
        },
        ast::Expr::As(subject, expected_typ) => {
            let expected_type_resolved = db.resolve_typ(expected_typ.clone(), moddef_id.package())?;
            let typed_subject = db.typecheck_expr(moddef_id.clone(), subject.clone(), expected_type_resolved.clone(), ctx)?;
            if expected_type_resolved != typ {
                return Err(VirdantError::Unknown);
            } else {
                Ok(TypedExpr::As(expected_type_resolved, typed_subject.clone(), expected_typ.clone()).into())
            }
        },
        ast::Expr::Idx(_subject, _i) => {
            let typed_expr = db.typeinfer_expr(moddef_id, expr, ctx)?;
            if typed_expr.typ() != typ {
                Err(VirdantError::Unknown)
            } else {
                Ok(typed_expr)
            }
        },
        ast::Expr::IdxRange(_subject, _j, _i) => {
            let typed_expr = db.typeinfer_expr(moddef_id, expr, ctx)?;
            if typed_expr.typ() != typ {
                Err(VirdantError::Unknown)
            } else {
                Ok(typed_expr)
            }
        },
        ast::Expr::Cat(_) => todo!(),
        ast::Expr::If(c, a, b) => {
            let typed_c = db.typecheck_expr(moddef_id.clone(), c.clone(), Type::Word(1), ctx.clone())?;
            let typed_a = db.typecheck_expr(moddef_id.clone(), a.clone(), typ.clone(), ctx.clone())?;
            let typed_b = db.typecheck_expr(moddef_id.clone(), b.clone(), typ.clone(), ctx.clone())?;
            Ok(TypedExpr::If(typ, typed_c, typed_a, typed_b).into())
        },
        ast::Expr::Let(x, ascription, e, b) => {
            let typed_e = match ascription {
                Some(ascribed_typ) => {
                    let resolved_ascribed_typ = db.resolve_typ(ascribed_typ.clone(), moddef_id.package())?;
                    db.typecheck_expr(moddef_id.clone(), e.clone(), resolved_ascribed_typ, ctx.clone())?
                },
                None => db.typeinfer_expr(moddef_id.clone(), e.clone(), ctx.clone())?,
            };

            let new_ctx = ctx.extend(x.clone(), typed_e.typ());
            let typed_b = db.typecheck_expr(moddef_id, b.clone(), typ.clone(), new_ctx)?;
            Ok(TypedExpr::Let(typed_b.typ(), x.clone(), ascription.clone(), typed_e, typed_b).into())
        },
        ast::Expr::Match(subject, ascription, arms) => {
            let typed_subject = db.typeinfer_expr(moddef_id.clone(), subject.clone(), ctx.clone())?;

            let uniondef_id = match typed_subject.typ() {
                Type::Union(uniondef_id, _typeargs) => uniondef_id,
                _ => return Err(virdant_error!("Can only match against a union type")),
            };


            let ctor_element_ids  = db.item_elements(uniondef_id.as_item())?;
            let ctors: Vec<Ident> = ctor_element_ids.iter().map(|element| element.clone().name()).collect();

            let mut typed_arms: Vec<TypedMatchArm> = vec![];
            for ast::MatchArm(pat, e) in arms {
                let mut new_ctx = ctx.clone();
                match pat {
                    ast::Pat::At(ctor, subpats) => {
                        let CtorSig(arg_typs, _typ) = db.ctor_sig(typed_subject.typ(), ctor.clone())?;

                        if subpats.len() != arg_typs.len() {
                            return Err(VirdantError::Unknown);
                        }

                        for (subpat, arg_typ) in subpats.iter().zip(arg_typs) {
                            if let ast::Pat::Bind(x) = subpat {
                                eprintln!("Extending ctx with {x} : {arg_typ}");
                                new_ctx = new_ctx.extend(x.clone(), arg_typ);
                            } else {
                                return Err(VirdantError::Unknown);
                            }
                        }
                    },
                    ast::Pat::Bind(x) => todo!(),
                    ast::Pat::Otherwise => todo!(),
                }
                let typed_e = db.typecheck_expr(moddef_id.clone(), e.clone(), typ.clone(), new_ctx)?;
                let typed_pat = TypedPat::from(pat, typed_subject.typ(), db)?;
                let typed_arm = TypedMatchArm(typed_pat, typed_e);
                typed_arms.push(typed_arm);
            }
            // TODO type ascription
            Ok(TypedExpr::Match(typ.clone(), typed_subject, None, typed_arms).into())
        },
    }
}

fn typeinfer_expr(
    db: &dyn TypecheckQ,
    moddef_id: ModDefId,
    expr: Arc<ast::Expr>,
    ctx: Context<Ident, Type>,
) -> VirdantResult<Arc<TypedExpr>> {
    match expr.as_ref() {
        ast::Expr::Reference(path) => {
            // is it a local reference?
            if let Some(ident) = path.as_ident() {
                if let Some(actual_typ) = ctx.lookup(&ident) {
                    return Ok(TypedExpr::Reference(actual_typ, Referent::Local(ident)).into());
                } 
            }
            let actual_typ = db.moddef_reference_type(moddef_id.clone(), path.clone())?;
            let component_id: ComponentId = db.resolve_component(moddef_id.clone(), path.clone())?;

            if path.is_local() {
                Ok(TypedExpr::Reference(actual_typ, Referent::LocalComponent(component_id)).into())
            } else {
                let submodule_element_id = db.resolve_element(moddef_id.clone(), path.head())?;
                Ok(TypedExpr::Reference(actual_typ, Referent::NonLocalComponent(submodule_element_id, component_id)).into())
            }
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
            let typed_subject = db.typeinfer_expr(moddef_id.clone(), subject.clone(), ctx.clone())?;
            let MethodSig(arg_types, ret_typ) = db.method_sig(typed_subject.typ(), method.clone())?;

            if args.len() != arg_types.len() {
                return Err(VirdantError::Unknown);
            }

            let mut typed_args = vec![];
            for (arg, arg_type) in args.iter().zip(arg_types) {
                let typed_arg = db.typecheck_expr(moddef_id.clone(), arg.clone(), arg_type, ctx.clone())?;
                typed_args.push(typed_arg);
            }

            Ok(TypedExpr::MethodCall(ret_typ, typed_subject, method.clone(), typed_args).into())
        },
        ast::Expr::Ctor(_ctor, _args) => {
            Err(TypeError::CantInfer.into())
        },
        ast::Expr::As(_, _) => todo!(),
        ast::Expr::Idx(subject, i) => {
            eprintln!("TODO: Check i fits in the size of the subject");
            let typed_subject = db.typeinfer_expr(moddef_id.clone(), subject.clone(), ctx)?;
            Ok(TypedExpr::Idx(Type::Word(1), typed_subject, *i).into())
        },
        ast::Expr::IdxRange(subject, j, i) => {
            eprintln!("TODO: Check i fits in the size of the subject");
            let typed_subject = db.typeinfer_expr(moddef_id.clone(), subject.clone(), ctx)?;
            Ok(TypedExpr::IdxRange(Type::Word(j - i), typed_subject, *j, *i).into())
        },
        ast::Expr::Cat(_) => todo!(),
        ast::Expr::If(_, _, _) => Err(VirdantError::Other("Can't infer".to_string())),
        ast::Expr::Let(x, ascription, e, b) => {
            let typed_e = match ascription {
                Some(ascribed_typ) => {
                    let resolved_ascribed_typ = db.resolve_typ(ascribed_typ.clone(), moddef_id.package())?;
                    db.typecheck_expr(moddef_id.clone(), e.clone(), resolved_ascribed_typ, ctx.clone())?
                },
                None => db.typeinfer_expr(moddef_id.clone(), e.clone(), ctx.clone())?,
            };

            let new_ctx = ctx.extend(x.clone(), typed_e.typ());
            let typed_b = db.typeinfer_expr(moddef_id, b.clone(), new_ctx)?;
            Ok(TypedExpr::Let(typed_b.typ(), x.clone(), ascription.clone(), typed_e, typed_b).into())
        },
        ast::Expr::Match(_subject, _ascription, _arms) => Err(TypeError::CantInfer.into()),
    }
}

fn moddef_reference_type(db: &dyn TypecheckQ, moddef_id: ModDefId, path: Path) -> VirdantResult<Type> {
    eprintln!("moddef_reference_type({moddef_id}, {path})");
    // moddef_reference_type(top::Top, top::Top::edge_detector.out)
    let moddef_ast = db.moddef_ast(moddef_id.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(c) if c.name.as_path() == path => {
                let typ = db.resolve_typ(c.typ.clone(), moddef_id.package())?;
                return Ok(typ);
            },
            ast::Decl::Submodule(submodule) if submodule.name.as_path() == path.parent() => {
                let submodule_moddef_id = db.moddef(submodule.moddef.clone().into(), moddef_id.package())?;
                return db.moddef_reference_type(submodule_moddef_id, path.name().as_path());
            },
            _ => (),
        }
    }

    Err(VirdantError::Other(format!("Component not found: `{path}` in `{moddef_id}`")))
}

fn typecheck_moddef(_db: &dyn TypecheckQ, _moddef: ModDefId) -> VirdantResult<()> {
    todo!()
}

fn typecheck(_db: &dyn TypecheckQ, _moddef: ModDefId) -> VirdantResult<()> {
    todo!()
}
