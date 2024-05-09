use crate::common::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprIdx(pub Expr, pub StaticIndex);

impl ExprIdx {
    pub fn subject(&self) -> Expr {
        self.0.clone()
    }

    pub fn index(&self) -> StaticIndex {
        self.1
    }
}

impl IsExpr for ExprIdx {
    fn subexprs(&self) -> Vec<Expr> {
        vec![self.0.clone()]
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        let typed_subject: Expr = self.subject().typeinfer(ctx.clone())?;
        let subject_type: Arc<Type> = typed_subject.type_of().unwrap();
        if let Type::Word(n) = subject_type.as_ref() {
            if self.index() < *n {
                let typ: Arc<Type> = Type::Word(1).into();
                Ok(ExprNode::Idx(ExprIdx(typed_subject, self.index())).with_type(typ))
            } else {
                Err(TypeError::Unknown)
            }
        } else {
            Err(TypeError::Unknown)
        }
    }

    fn eval(&self, ctx: Context<Path, Value>, typ: Arc<Type>) -> Value {
        let subject_value = self.subject().eval(ctx);
        if let Value::Word(_width, v) = subject_value {
            // v_index_masked should be 0b00..000100..00
            // eg, 1 in position self.index(), or else all zeroes
            let v_index_masked = v & (1 << self.index());
            // v_new should be 0b000.....01
            // eg, 1 in index 0, or else all zeroes
            let v_new = v_index_masked >> self.index();
            assert!(v_new == 0 || v_new == 1);
            Value::Word(1, v_new)
        } else {
            Value::X(typ.clone())
        }
    }
}
