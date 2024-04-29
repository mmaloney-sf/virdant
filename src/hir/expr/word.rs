use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprWord(pub WordLit);

impl IsExpr for ExprWord {
    fn subexprs(&self) -> Vec<Expr> {
        vec![]
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        if let Some(width) = self.width() {
            let typ = Arc::new(Type::Word(width));
            Ok(ExprNode::Word(self.clone()).with_type(typ))
        } else {
            Err(TypeError::CantInfer)
        }
    }

    fn typecheck(&self, ctx: Context<Path, Arc<Type>>, type_expected: Arc<Type>) -> Result<Expr, TypeError> {
        todo!()
        /*
        match (type_expected.as_ref(), self.width()) {
            (Type::Word(expected_width), Some(actual_width)) if *expected_width == actual_width => {
                let typ = Arc::new(Type::Word(actual_width));
                Ok(())
            },
            (Type::Word(expected_width), None) if fits_in(self.value(), *expected_width) => {
                let typ = Arc::new(Type::Word(*expected_width));
                self.typecell().set(typ.clone());
                Ok(())
            },
            (Type::Word(expected_width), None) =>  Err(TypeError::Other),
            (_, _) => Err(TypeError::Other),
        }
        */
    }

    fn eval(&self, _ctx: Context<Path, Value>) -> Value {
        Value::Word(self.width().unwrap(), self.value())
    }
}

impl ExprWord {
    fn value(&self) -> u64 {
        self.0.value
    }
    fn width(&self) -> Option<Width> {
        self.0.width
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
