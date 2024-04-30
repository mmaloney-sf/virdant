mod ascription;
mod reference;
mod word;
mod vec;
mod methodcall;

use ascription::*;
use reference::*;
use word::*;
use vec::*;
use methodcall::*;

use std::collections::HashSet;

use crate::ast;
use crate::value::Value;
use crate::common::*;
use crate::context::Context;
use super::Type;

#[derive(Debug, Clone)]
pub struct Expr(Option<Arc<Type>>, Arc<ExprNode>);

impl Expr {
    fn as_node(&self) -> &ExprNode {
        &self.1
    }

    pub fn type_of(&self) -> Option<Arc<Type>> {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
pub enum ExprNode {
    Reference(ExprReference),
    Word(ExprWord),
    Vec(ExprVec),
    As(ExprAs),
    MethodCall(ExprMethodCall),
}

impl Expr {
    fn to_class(&self) -> &dyn IsExpr {
        match self.as_node() {
            ExprNode::Reference(inner) => inner,
            ExprNode::Word(inner) => inner,
            ExprNode::MethodCall(inner) => inner,
            ExprNode::Vec(inner) => inner,
            ExprNode::As(inner) => inner,
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
                Err(TypeError::TypeMismatch())
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
            _ => todo!(),
        };
        expr_node.without_type()
    }
}
