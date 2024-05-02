use crate::common::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprVec(pub Vec<Expr>);

impl IsExpr for ExprVec {
    fn subexprs(&self) -> Vec<Expr> {
        self.0.clone()
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        if self.0.len() == 0 {
            Err(TypeError::CantInfer)
        } else {
            let typed_args: Vec<Expr> = self.0.iter().map(|arg| arg.typeinfer(ctx.clone()).unwrap()).collect();
            let element_type = typed_args[0].type_of().unwrap();
            for typed_arg in &typed_args {
                if typed_arg.type_of().unwrap() != element_type {
                    return Err(TypeError::Other);
                }
            }
            let typ: Arc<Type> = Type::Vec(element_type, self.0.len()).into();
            Ok(ExprNode::Vec(ExprVec(typed_args)).with_type(typ))
        }
    }

    fn eval(&self, ctx: Context<Path, Value>, typ: Arc<Type>) -> Value {
        let vs = self.0.iter().map(|e| e.eval(ctx.clone())).collect::<Vec<Value>>();
        Value::Vec(typ.clone(), vs)
    }
}
