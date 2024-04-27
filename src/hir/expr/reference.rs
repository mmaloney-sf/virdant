use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprReference(pub Option<Arc<Type>>, pub Path);

impl ExprReference {
    pub fn path(&self) -> &Path {
        &self.1
    }
}

impl IsExpr for ExprReference {
    fn subexprs(&self) -> Vec<Arc<Expr>> {
        vec![]
    }

    fn type_of(&self) -> Option<Arc<Type>> {
        self.0.clone()
    }

    fn references(&self) -> HashSet<Path> {
        vec![self.path().clone()].into_iter().collect()
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Option<Arc<Type>> {
        ctx.lookup(self.path())
    }
}
