use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprVec(pub Option<Arc<Type>>, pub Vec<Arc<Expr>>);

impl IsExpr for ExprVec {
    fn subexprs(&self) -> Vec<Arc<Expr>> {
        self.1.clone()
    }

    fn type_of(&self) -> Option<Arc<Type>> {
        self.0.clone()
    }

    fn typeinfer(&self, _ctx: Context<Path, Arc<Type>>) -> Option<Arc<Type>> {
        todo!()
    }

}
