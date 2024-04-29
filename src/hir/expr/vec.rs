use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprVec(pub Vec<Expr>);

impl IsExpr for ExprVec {
    fn subexprs(&self) -> Vec<Expr> {
        self.0.clone()
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        todo!()
    }
}
