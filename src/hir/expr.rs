mod reference;
mod word;
mod vec;
mod methodcall;

use reference::*;
use word::*;
use vec::*;
use methodcall::*;

use std::sync::RwLock;
use std::collections::HashSet;

use crate::ast;
use crate::value::Value;
use crate::common::*;
use crate::types::Type;
use crate::context::Context;
use crate::ast::WordLit;

#[derive(Debug, Clone)]
pub enum Expr {
    Reference(ExprReference),
    Word(ExprWord),
    Vec(ExprVec),
    MethodCall(ExprMethodCall),
}

impl Expr {
    fn to_class(&self) -> &dyn IsExpr {
        match self {
            Expr::Reference(inner) => inner,
            Expr::Word(inner) => inner,
            Expr::MethodCall(inner) => inner,
            Expr::Vec(inner) => inner,
        }
    }
}

pub trait IsExpr {
    fn subexprs(&self) -> Vec<Arc<Expr>>;

    fn references(&self) -> HashSet<Path> {
        let mut result = HashSet::new();
        for e in self.subexprs() {
            result.extend(e.to_class().references().into_iter());
        }
        result
    }

    fn type_of(&self) -> Arc<Type> {
        self.typecell().get().clone()
    }

    fn typecell(&self) -> TypeCell;

    fn typeinfer(&self, _ctx: Context<Path, Arc<Type>>) -> Result<Arc<Type>, TypeError> {
        Err(TypeError::CantInfer)
    }

    fn typecheck(&self, ctx: Context<Path, Arc<Type>>, type_expected: Arc<Type>) -> Result<(), TypeError> {
        if let Ok(type_actual) = self.typeinfer(ctx) {
            if type_actual == type_expected {
                Ok(())
            } else {
                Err(TypeError::TypeMismatch())
            }
        } else {
            Err(TypeError::CantInfer)
        }
    }

    fn eval(&self, ctx: Context<Path, Value>) -> Value {
        todo!()
    }
}

impl IsExpr for Expr {
    fn subexprs(&self) -> Vec<Arc<Expr>> { self.to_class().subexprs() }
    fn typecell(&self) -> TypeCell { self.to_class().typecell() }
    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Arc<Type>, TypeError> { self.to_class().typeinfer(ctx) }
    fn typecheck(&self, ctx: Context<Path, Arc<Type>>, type_expected: Arc<Type>) -> Result<(), TypeError> { self.to_class().typecheck(ctx, type_expected) }
}

impl Expr {
    pub fn to_hir(expr: &ast::Expr) -> Result<Arc<Expr>, VirdantError> {
        match expr {
            ast::Expr::Reference(path) => Ok(Expr::Reference(ExprReference(TypeCell::unknown(), path.clone())).into()),
            ast::Expr::Word(lit) => Ok(Expr::Word(ExprWord(TypeCell::unknown(), lit.clone())).into()),
            ast::Expr::Vec(es) => {
                let mut es_hir = vec![];
                for e in es {
                    es_hir.push(Expr::to_hir(e)?);
                }
                Ok(Expr::Vec(ExprVec(TypeCell::unknown(), es_hir)).into())
            },
            ast::Expr::MethodCall(subject, method, args) => {
                let subject_hir = Expr::to_hir(subject)?;
                let mut args_hir = vec![];
                for arg in args {
                    args_hir.push(Expr::to_hir(arg)?);
                }
                Ok(Expr::MethodCall(ExprMethodCall(TypeCell::unknown(), subject_hir, method.clone(), args_hir)).into())
            },
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeCell(Arc<RwLock<Option<Arc<Type>>>>);

impl TypeCell {
    fn unknown() -> TypeCell {
        TypeCell(Arc::new(RwLock::new(None)))
    }

    fn get(&self) -> Arc<Type> {
        let lock = self.0.read().unwrap();
        lock.clone().unwrap()
    }

    fn set(&self, typ: Arc<Type>) {
        let mut lock = self.0.write().unwrap();
        assert!(lock.is_none(), "TypeCell is already set");
        *lock = Some(typ);
    }
}

#[test]
fn test_parse_exprs() {
    use crate::ast;
    use crate::parse;

    let expr_strs = vec![
        "0",
        "1_000",
        "0b1010",
        "2w8",
        "0b1010w4",
//        "0xff",
//        "0xffw16",
        "x",
        "x.y",
        "x.y.z",
        "[]",
        "[0, 1, 1, 2, 3, 5]",
        "(x)",
//        "cat(x, y, z)",
//        "f(x, y)",
        "z->f(x, y)",
        "a->eq(b)",
        "a->lt(b)",
        "a->lte(b)",
//        "x->real",
//        "x[0]",
//        "x[8..0]",
//        "x[i]",
//        "struct Unit {}",
//        "struct Complex { real = 0w8, imag = 1w8 }",
//        "struct Complex { real = 0w8, imag = 1w8 }",
        /*
        "
            with x {
                this[0] = 1w8;
                this[2] = 7w8;
            }
        ",
        */
    ];
    for expr_str in expr_strs {
        eprintln!("Testing {expr_str:?}");
        let expr: ast::Expr = parse::parse_expr(expr_str).unwrap();
        Expr::to_hir(&expr).unwrap();
    }
}
