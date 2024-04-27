use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprMethodCall(pub Arc<Expr>, pub Ident, pub Vec<Arc<Expr>>);

impl IsExpr for ExprMethodCall {
    fn subexprs(&self) -> Vec<Arc<Expr>> {
        vec![]
    }
}
