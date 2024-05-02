use crate::common::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprAs(pub Expr, pub Arc<Type>);

impl IsExpr for ExprAs {
    fn subexprs(&self) -> Vec<Expr> {
        vec![self.0.clone()]
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        let typ: Arc<Type> = self.1.clone();
        let inner = self.0.typecheck(ctx, typ.clone())?;
        Ok(ExprNode::As(ExprAs(inner, typ.clone())).with_type(typ))
    }

    fn eval(&self, ctx: Context<Path, Value>, _typ: Arc<Type>) -> Value {
        self.0.eval(ctx)
    }
}
