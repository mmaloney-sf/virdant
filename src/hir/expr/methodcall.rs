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
            (Type::Word(n), method@("eq" | "lt" | "gt" | "lte" | "gte")) => {
                assert_eq!(args.len(), 1);
                let arg: Expr = args[0].clone();
                let typed_arg = arg.typecheck(ctx.clone(),  Type::Word(*n).into())?;
                let typ: Arc<Type> = Type::Word(1).into();
                Ok(ExprNode::MethodCall(ExprMethodCall(typed_subject, method.into(), vec![typed_arg])).with_type(typ))
            },
            (Type::Word(n), method@("add" | "sub" | "and" | "or")) => {
                assert_eq!(args.len(), 1);
                let arg: Expr = args[0].clone();
                let typed_arg = arg.typecheck(ctx.clone(),  Type::Word(*n).into())?;
                let typ: Arc<Type> = Type::Word(*n).into();
                Ok(ExprNode::MethodCall(ExprMethodCall(typed_subject, method.into(), vec![typed_arg])).with_type(typ))
            },
            (Type::Word(n), method@("not" | "neg")) => {
                assert_eq!(args.len(), 0);
                let typ: Arc<Type> = Type::Word(*n).into();
                Ok(ExprNode::MethodCall(ExprMethodCall(typed_subject, method.into(), vec![])).with_type(typ))
            },
            _ => panic!(),
        }
    }

    fn eval(&self, ctx: Context<Path, Value>, typ: Arc<Type>) -> Value {
        let ExprMethodCall(subject, name, args) = self;
        let subject_value: Value = subject.eval(ctx.clone());
        let arg_values: Vec<Value> = args.iter().map(|arg| arg.eval(ctx.clone())).collect();
        let subject_type = &*subject.type_of().unwrap();

        let result_value = match (subject_type, name.as_str()) {
            (Type::Word(_n), "eq") => {
                let v = (subject_value.unwrap_word() == arg_values.first().unwrap().unwrap_word()) as u64;
                Value::Word(1, v)
            },
            (Type::Word(_n), "lt") => {
                let v = (subject_value.unwrap_word() < arg_values.first().unwrap().unwrap_word()) as u64;
                Value::Word(1, v)
            },
            (Type::Word(_n), "lte") => {
                let v = (subject_value.unwrap_word() <= arg_values.first().unwrap().unwrap_word()) as u64;
                Value::Word(1, v)
            },
            (Type::Word(_n), "gt") => {
                let v = (subject_value.unwrap_word() > arg_values.first().unwrap().unwrap_word()) as u64;
                Value::Word(1, v)
            },
            (Type::Word(_n), "gte") => {
                let v = (subject_value.unwrap_word() >= arg_values.first().unwrap().unwrap_word()) as u64;
                Value::Word(1, v)
            },
            (Type::Word(n), "and") => {
                let v = subject_value.unwrap_word() & arg_values.first().unwrap().unwrap_word();
                Value::Word(*n, v)
            },
            (Type::Word(n), "or") => {
                let v = subject_value.unwrap_word() | arg_values.first().unwrap().unwrap_word();
                Value::Word(*n, v)
            },
            (Type::Word(n), "add") => {
                let a = subject_value.unwrap_word();
                let b = arg_values.first().unwrap().unwrap_word();
                Value::Word(*n, a.wrapping_add(b) % (1 << n))
            },
            (Type::Word(n), "sub") => {
                let a = subject_value.unwrap_word();
                let b = arg_values.first().unwrap().unwrap_word();
                Value::Word(*n, a.wrapping_sub(b) % (1 << n))
            },
            (Type::Word(n), "not") => {
                let a = subject_value.unwrap_word();
                Value::Word(*n, !a % (1 << n))
            },
            (Type::Word(n), "neg") => {
                let a = subject_value.unwrap_word();
                Value::Word(*n, !a % (1 << n))
            },
            _ => panic!(),
        };
        assert_eq!(result_value.type_of(), typ);
        result_value
    }
}
