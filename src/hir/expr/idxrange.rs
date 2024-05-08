use crate::common::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprIdxRange(pub Expr, pub StaticIndex, pub StaticIndex);

impl ExprIdxRange {
    fn subject(&self) -> Expr {
        self.0.clone()
    }

    fn index0(&self) -> StaticIndex {
        self.1
    }

    fn index1(&self) -> StaticIndex {
        self.2
    }
}

impl IsExpr for ExprIdxRange {
    fn subexprs(&self) -> Vec<Expr> {
        vec![self.0.clone()]
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        let typed_subject: Expr = self.subject().typeinfer(ctx.clone())?;
        let subject_type: Arc<Type> = typed_subject.type_of().unwrap();
        if let Type::Word(n) = subject_type.as_ref() {
            let idx_hi = self.index0();
            let idx_lo = self.index1();

            if idx_lo <= idx_hi && idx_hi <= *n {
                let typ: Arc<Type> = Type::Word(idx_hi - idx_lo).into();
                Ok(ExprNode::IdxRange(ExprIdxRange(typed_subject, idx_hi, idx_lo)).with_type(typ))
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
            let idx_hi = self.index0();
            let idx_lo = self.index1();

            // v_index_masked should be 0b00..000100..00
            // eg, 1 in position self.index(), or else all zeroes
            let v_index_masked = v & (1 << (idx_hi + 1)) - 1;
            // v_new should be 0b000.....01
            // eg, 1 in index 0, or else all zeroes
            let v_new = v_index_masked >> idx_lo;
            Value::Word(1, v_new)
        } else {
            Value::X(typ.clone())
        }
    }
}

