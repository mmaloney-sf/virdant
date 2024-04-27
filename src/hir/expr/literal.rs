use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub enum ExprLiteral {
    Word(WordLit),
}


impl IsExpr for ExprLiteral {
    fn subexprs(&self) -> Vec<Arc<Expr>> {
        vec![]
    }
}
