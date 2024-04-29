use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprMethodCall(pub Expr, pub Ident, pub Vec<Expr>);

impl IsExpr for ExprMethodCall {
    fn subexprs(&self) -> Vec<Expr> {
        vec![]
    }

    fn eval(&self, ctx: Context<Path, Value>) -> Value { todo!() }
}
