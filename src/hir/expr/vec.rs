use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprVec(pub TypeCell, pub Vec<Arc<Expr>>);

impl IsExpr for ExprVec {
    fn subexprs(&self) -> Vec<Arc<Expr>> {
        self.1.clone()
    }

    fn typecell(&self) -> TypeCell {
        self.0.clone()
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Arc<Type>, TypeError> {
        todo!()
    }

}
