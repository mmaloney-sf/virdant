use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprReference(pub TypeCell, pub Path);

impl ExprReference {
    pub fn path(&self) -> &Path {
        &self.1
    }
}

impl IsExpr for ExprReference {
    fn subexprs(&self) -> Vec<Arc<Expr>> {
        vec![]
    }

    fn typecell(&self) -> TypeCell {
        self.0.clone()
    }

    fn references(&self) -> HashSet<Path> {
        vec![self.path().clone()].into_iter().collect()
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Arc<Type>, TypeError> {
        if let Some(type_actual) = ctx.lookup(self.path()) {
            Ok(type_actual.clone())
        } else {
            Err(TypeError::CantInfer)
        }
    }
}
