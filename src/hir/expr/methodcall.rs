use crate::common::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprMethodCall(pub Expr, pub Ident, pub Vec<Expr>);

impl ExprMethodCall {
    pub fn subject(&self) -> Expr {
        self.0.clone()
    }

    pub fn method(&self) -> Ident {
        self.1.clone()
    }

    pub fn args(&self) -> Vec<Expr> {
        self.2.clone()
    }
}

impl IsExpr for ExprMethodCall {
    fn subexprs(&self) -> Vec<Expr> {
        let mut result = vec![self.0.clone()];
        result.extend(self.2.clone());
        result
    }

    fn typeinfer(&self, ctx: Context<Path, Arc<Type>>) -> Result<Expr, TypeError> {
        let ExprMethodCall(subject, method, args) = self;
        let typed_subject: Expr = subject.typeinfer(ctx.clone())?;
        let subject_type: &Type = &typed_subject.type_of().unwrap();

        match (subject_type, method.as_str()) {
            (Type::Word(n), "eq") => {
                assert_eq!(args.len(), 1);
                let arg: Expr = args[0].clone();
                let typed_arg = arg.typecheck(ctx.clone(),  Type::Word(*n).into())?;
                let typ: Arc<Type> = Type::Word(1).into();
                Ok(ExprNode::MethodCall(ExprMethodCall(typed_subject, "eq".into(), vec![typed_arg])).with_type(typ))
            },
            // 1w8->add(2)
            (Type::Word(n), "add") => {
                assert_eq!(args.len(), 1);
                let arg: Expr = args[0].clone();
                let typed_arg = arg.typecheck(ctx.clone(),  Type::Word(*n).into())?;
                let typ: Arc<Type> = Type::Word(*n).into();
                Ok(ExprNode::MethodCall(ExprMethodCall(typed_subject, "add".into(), vec![typed_arg])).with_type(typ))
            },
            _ => panic!(),
        }
    }

    fn eval(&self, ctx: Context<Path, Value>, _typ: Arc<Type>) -> Value {
        let ExprMethodCall(subject, name, args) = self;
        let subject_value: Value = subject.eval(ctx.clone());
        let arg_values: Vec<Value> = args.iter().map(|arg| arg.eval(ctx.clone())).collect();
        let subject_type = &*subject.type_of().unwrap();

        match (subject_type, name.as_str()) {
            (Type::Word(_n), "eq") => {
                Value::Bool(subject_value == *arg_values.first().unwrap())
            },
            (Type::Word(n), "add") => {
                let a = subject_value.unwrap_word();
                let b = arg_values.first().unwrap().unwrap_word();
                Value::Word(*n, a.wrapping_add(b) % (1 << n))
            },
            _ => panic!(),
        }
    }
}
