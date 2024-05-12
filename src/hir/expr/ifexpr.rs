use crate::common::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprIf(pub Box<Expr>, pub Box<Expr>, pub Box<Expr>);

impl IsExpr for ExprIf {
    fn subexprs(&self) -> Vec<Expr> {
        vec![
            *self.0.clone(),
            *self.1.clone(),
            *self.2.clone(),
        ]
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        let typed_condition: Expr = self.condition().typeinfer(ctx.clone())?;
        let condition_type: &Type = &typed_condition.type_of().unwrap();

        if let Type::Word(1) = condition_type {
            let typed_a: Expr = self.a().typeinfer(ctx.clone())?;
            let typed_b: Expr = self.b().typeinfer(ctx.clone())?;

            let typ = typed_a.type_of().unwrap();
            if typed_a.type_of().unwrap() == typed_b.type_of().unwrap() {
                Ok(ExprNode::If(ExprIf(Box::new(typed_condition), Box::new(typed_a), Box::new(typed_b))).with_type(typ))
            } else {
                return Err(TypeError::Unknown);
            }
        } else {
            return Err(TypeError::Unknown);
        }
    }

    fn eval(&self, ctx: Context<Path, Value>, typ: Arc<Type>) -> Value {
        let c = self.condition().eval(ctx.clone()).unwrap_word() == 1;
        if c {
            self.a().eval(ctx)
        } else {
            self.b().eval(ctx)
        }
    }
}

impl ExprIf {
    pub fn condition(&self) -> &Expr {
        &self.0
    }

    pub fn a(&self) -> &Expr {
        &self.1
    }

    pub fn b(&self) -> &Expr {
        &self.2
    }
}
