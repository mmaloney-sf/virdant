use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub enum ExprLiteral {
    Word(Option<Arc<Type>>, WordLit),
}


impl IsExpr for ExprLiteral {
    fn subexprs(&self) -> Vec<Arc<Expr>> {
        vec![]
    }

    fn type_of(&self) -> Option<Arc<Type>> {
        match self {
            ExprLiteral::Word(typ, _lit) => typ.clone(),
        }
    }
}

fn fits_in(value: u64, width: Width) -> bool {
    if width > 63 {
        false
    } else {
        value < (1 << width)
    }
}
