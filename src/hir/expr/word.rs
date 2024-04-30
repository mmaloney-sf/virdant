use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprWord(pub u64, pub Option<Width>);

impl IsExpr for ExprWord {
    fn subexprs(&self) -> Vec<Expr> {
        vec![]
    }

    fn typeinfer(&self, _ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        if let Some(width) = self.width() {
            let typ = Arc::new(Type::Word(width));
            Ok(ExprNode::Word(self.clone()).with_type(typ))
        } else {
            Err(TypeError::CantInfer)
        }
    }

    fn typecheck(&self, _ctx: Context<Path, Arc<Type>>, type_expected: Arc<Type>) -> Result<Expr, TypeError> {
        match (type_expected.as_ref(), self.width()) {
            (Type::Word(expected_width), Some(actual_width)) if *expected_width == actual_width => {
                let typ = Arc::new(Type::Word(actual_width));
                Ok(ExprNode::Word(self.clone()).with_type(typ))
            },
            (Type::Word(expected_width), None) if fits_in(self.value(), *expected_width) => {
                let typ = Arc::new(Type::Word(*expected_width));
                Ok(ExprNode::Word(ExprWord(self.value(), Some(*expected_width))).with_type(typ))
            },
            (Type::Word(_expected_width), None) =>  Err(TypeError::Other),
            (_, _) => Err(TypeError::Other),
        }
    }

    fn eval(&self, _ctx: Context<Path, Value>, _typ: Arc<Type>) -> Value {
        Value::Word(self.width().unwrap(), self.value())
    }
}

impl ExprWord {
    fn value(&self) -> u64 {
        self.0
    }
    fn width(&self) -> Option<Width> {
        self.1
    }
}

fn fits_in(value: u64, width: Width) -> bool {
    // TODO
    if width > 63 {
        false
    } else {
        value < (1 << width)
    }
}
