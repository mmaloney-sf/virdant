mod ascription;
mod reference;
mod word;
mod vec;
mod methodcall;
mod idx;
mod idxrange;
mod cat;
mod ifexpr;

use ascription::*;
use reference::*;
use word::*;
use vec::*;
use methodcall::*;
use idx::*;
use idxrange::*;
use cat::*;
use ifexpr::*;

use std::collections::HashSet;

use crate::ast;
use crate::value::Value;
use crate::common::*;
use crate::context::Context;
use super::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr(Option<Arc<Type>>, Arc<ExprNode>);

impl Expr {
    pub fn as_node(&self) -> &ExprNode {
        &self.1
    }

    pub fn type_of(&self) -> Option<Arc<Type>> {
        self.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprNode {
    Reference(ExprReference),
    Word(ExprWord),
    Vec(ExprVec),
    As(ExprAs),
    MethodCall(ExprMethodCall),
    Idx(ExprIdx),
    IdxRange(ExprIdxRange),
    Cat(ExprCat),
    If(ExprIf),
}

impl Expr {
    fn to_class(&self) -> &dyn IsExpr {
        match self.as_node() {
            ExprNode::Reference(inner) => inner,
            ExprNode::Word(inner) => inner,
            ExprNode::MethodCall(inner) => inner,
            ExprNode::Vec(inner) => inner,
            ExprNode::As(inner) => inner,
            ExprNode::Idx(inner) => inner,
            ExprNode::IdxRange(inner) => inner,
            ExprNode::Cat(inner) => inner,
            ExprNode::If(inner) => inner,
        }
    }
}

impl ExprNode {
    pub fn without_type(self) -> Expr {
        Expr(None, self.into())
    }

    pub fn with_type(self, typ: Arc<Type>) -> Expr {
        Expr(Some(typ), self.into())
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
                Err(TypeError::TypeMismatch(type_actual, type_expected))
            }
        } else {
            Err(TypeError::CantInfer)
        }
    }

    fn eval(&self, ctx: Context<Path, Value>, typ: Arc<Type>) -> Value;
}

impl Expr {
    pub fn references(&self) -> HashSet<Path> { self.to_class().references() }
    pub fn subexprs(&self) -> Vec<Expr> { self.to_class().subexprs() }
    pub fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> { self.to_class().typeinfer(ctx) }
    pub fn typecheck(&self, ctx: Context<Path, Arc<Type>>, type_expected: Arc<Type>) -> Result<Expr, TypeError> { self.to_class().typecheck(ctx, type_expected) }
    pub fn eval(&self, ctx: Context<Path, Value>) -> Value { self.to_class().eval(ctx, self.type_of().unwrap()) }
}

impl Expr {
    pub fn from_ast(expr: &ast::Expr) -> Expr {
        let expr_node = match expr {
            ast::Expr::Reference(path) => ExprNode::Reference(ExprReference(path.clone())),
            ast::Expr::Word(lit) => ExprNode::Word(ExprWord(lit.value, lit.width)),
            ast::Expr::Vec(es) => {
                let mut es_hir = vec![];
                for e in es {
                    es_hir.push(Expr::from_ast(e));
                }
                ExprNode::Vec(ExprVec(es_hir))
            },
            ast::Expr::MethodCall(subject, method, args) => {
                let subject_hir: Expr = Expr::from_ast(subject);
                let mut args_hir: Vec<Expr> = vec![];
                for arg in args {
                    args_hir.push(Expr::from_ast(arg));
                }
                ExprNode::MethodCall(ExprMethodCall(subject_hir, method.clone(), args_hir))
            },
            ast::Expr::As(subject, typ) => {
                let subject_hir: Expr = Expr::from_ast(subject);
                ExprNode::As(ExprAs(subject_hir, Type::from_ast(typ)))
            },
            ast::Expr::Idx(subject, idx) => {
                let subject_hir: Expr = Expr::from_ast(subject);
                ExprNode::Idx(ExprIdx(subject_hir, *idx))
            },
            ast::Expr::IdxRange(subject, idx0, idx1) => {
                let subject_hir: Expr = Expr::from_ast(subject);
                ExprNode::IdxRange(ExprIdxRange(subject_hir, *idx0, *idx1))
            },
            ast::Expr::Cat(es) => {
                let mut es_hir = vec![];
                for e in es {
                    es_hir.push(Expr::from_ast(e));
                }
                ExprNode::Cat(ExprCat(es_hir))
            },
            ast::Expr::If(cond, a, b) => {
                let cond_hir: Expr = Expr::from_ast(cond);
                let a_hir: Expr = Expr::from_ast(a);
                let b_hir: Expr = Expr::from_ast(b);
                ExprNode::If(ExprIf(Box::new(cond_hir), Box::new(a_hir), Box::new(b_hir)))
            },
            _ => todo!(),
        };
        expr_node.without_type()
    }
}
