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
pub struct Expr(Option<Arc<Type>>, Arc<ExprNode>);

impl Expr {
    fn as_node(&self) -> &ExprNode {
        &self.1
    }

    fn type_of(&self) -> Option<Arc<Type>> {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
pub enum ExprNode {
    Reference(ExprReference),
    Word(ExprWord),
    Vec(ExprVec),
    MethodCall(ExprMethodCall),
}

impl Expr {
    fn to_class(&self) -> &dyn IsExpr {
        match self.as_node() {
            ExprNode::Reference(inner) => inner,
            ExprNode::Word(inner) => inner,
            ExprNode::MethodCall(inner) => inner,
            ExprNode::Vec(inner) => inner,
        }
    }
}

impl ExprNode {
    pub fn to_expr(self) -> Expr {
        Expr(None, self.into())
    }
}

pub trait IsExpr {
    fn subexprs(&self) -> Vec<Expr>;

    fn references(&self) -> HashSet<Path> {
        let mut result = HashSet::new();
        for e in self.subexprs() {
            result.extend(e.to_class().references().into_iter());
        }
        result
    }

    fn typeinfer(&self, _ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        Err(TypeError::CantInfer)
    }

    fn typecheck(&self, ctx: Context<Path, Arc<Type>>, type_expected: Arc<Type>) -> Result<Expr, TypeError> {
        if let Ok(self_inferred) = self.typeinfer(ctx) {
            let type_actual = self_inferred.type_of().unwrap();
            if type_actual == type_expected {
                Ok(self_inferred)
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
    fn subexprs(&self) -> Vec<Expr> { self.to_class().subexprs() }
    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> { self.to_class().typeinfer(ctx) }
    fn typecheck(&self, ctx: Context<Path, Arc<Type>>, type_expected: Arc<Type>) -> Result<Expr, TypeError> { self.to_class().typecheck(ctx, type_expected) }
}

impl Expr {
    pub fn to_hir(expr: &ast::Expr) -> Result<Expr, VirdantError> {
        let expr_node = match expr {
            ast::Expr::Reference(path) => ExprNode::Reference(ExprReference(path.clone())),
            ast::Expr::Word(lit) => ExprNode::Word(ExprWord(lit.clone())),
            ast::Expr::Vec(es) => {
                let mut es_hir = vec![];
                for e in es {
                    es_hir.push(Expr::to_hir(e)?);
                }
                ExprNode::Vec(ExprVec(es_hir))
            },
            ast::Expr::MethodCall(subject, method, args) => {
                let subject_hir: Expr = Expr::to_hir(subject)?;
                let mut args_hir: Vec<Expr> = vec![];
                for arg in args {
                    args_hir.push(Expr::to_hir(arg)?);
                }
                ExprNode::MethodCall(ExprMethodCall(subject_hir, method.clone(), args_hir))
            },
            _ => todo!(),
        };
        Ok(expr_node.to_expr())
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
