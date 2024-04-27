use crate::common::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ExprWord(pub TypeCell, pub WordLit);

impl IsExpr for ExprWord {
    fn subexprs(&self) -> Vec<Arc<Expr>> {
        vec![]
    }

    fn typecell(&self) -> TypeCell {
        self.0.clone()
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Arc<Type>, TypeError> {
        todo!()
    }

    fn typecheck(&self, ctx: Context<Path, Arc<Type>>, typ: Arc<Type>) -> Result<(), TypeError> {
        /*
        if let Type::Word(n) = typ {
            if let Some(width) = self.width() {
                assert_eq!(*width, n);
                if *width == n {
                    typeinfer(ctx.clone(), expr)
                }
            } else {
                if self.fits() {
                    Expr::Word(Some(n), self.value())
                } else {
                    Err(TypeError::Other)
                }
            }
        } else {
            Err(TypeError::Other)
        }
        */
        todo!()
    }
}

impl ExprWord {
    fn value(&self) -> u64 {
        self.1.value
    }
    fn width(&self) -> Option<Width> {
        self.1.width
    }

    fn fits(&self) -> bool {
        // TODO
        if self.width().unwrap() > 63 {
            false
        } else {
            self.value() < (1 << self.width().unwrap())
        }
    }
}
