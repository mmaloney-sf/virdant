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

fn fits_in(value: u64, width: Width) -> bool {
    if width > 63 {
        false
    } else {
        value < (1 << width)
    }
}
