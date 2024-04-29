use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprMethodCall(pub Expr, pub Ident, pub Vec<Expr>);

impl IsExpr for ExprMethodCall {
    fn subexprs(&self) -> Vec<Expr> {
        vec![]
    }
}
