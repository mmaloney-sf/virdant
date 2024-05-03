use crate::common::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprCat(pub Vec<Expr>);

impl IsExpr for ExprCat {
    fn subexprs(&self) -> Vec<Expr> {
        self.0.clone()
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        let mut args = vec![];
        let mut w = 0u64;
        for e in &self.0 {
            let arg: Expr = e.typeinfer(ctx.clone())?;
            let arg_typ: Arc<Type> = arg.type_of().unwrap();
            args.push(arg);
            if let Type::Word(m) = arg_typ.as_ref() {
                w += m;
            } else {
                return Err(TypeError::Unknown);
            }
        }
        let typ: Arc<Type> = Type::Word(w).into();
        Ok(ExprNode::Cat(ExprCat(args)).with_type(typ))
    }

    /*
            let typed_args: Vec<Expr> = self.0.iter().map(|arg| arg.typeinfer(ctx.clone()).unwrap()).collect();
            let element_type = typed_args[0].type_of().unwrap();
            for typed_arg in &typed_args {
                if typed_arg.type_of().unwrap() != element_type {
                    return Err(TypeError::Unknown);
                }
            }
            let typ: Arc<Type> = Type::Vec(element_type, self.0.len()).into();
            Ok(ExprNode::Vec(ExprVec(typed_args)).with_type(typ))
    */

    fn eval(&self, ctx: Context<Path, Value>, typ: Arc<Type>) -> Value {
        let mut cat_width: u64 = 0;
        let mut cat_val: u64 = 0;
        let values = self.0.iter().map(|e| e.eval(ctx.clone()));

        for v in values.rev() {
            if let Value::Word(width, val) = v {
                cat_val |= val << cat_width;
                cat_width += width;
            } else {
                return Value::X(typ.clone());
            }
        }

        Value::Word(cat_width, cat_val)
    }
}

