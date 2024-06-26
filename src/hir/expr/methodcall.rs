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
            (Type::Word(1), method@"mux") => {
                assert_eq!(args.len(), 2);
                let arg0: Expr = args[0].clone();
                let arg1: Expr = args[1].clone();
                let typed_arg0 = arg0.typeinfer(ctx.clone())?;
                let typed_arg1 = arg1.typeinfer(ctx.clone())?;
                if typed_arg0.type_of().unwrap() != typed_arg1.type_of().unwrap() {
                    return Err(TypeError::Unknown);
                }
                let typ: Arc<Type> = typed_arg0.type_of().unwrap();
                Ok(ExprNode::MethodCall(ExprMethodCall(typed_subject, method.into(), vec![typed_arg0, typed_arg1])).with_type(typ))
            },
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
            (Type::Word(n), method@("sll" | "srl")) => {
                assert_eq!(args.len(), 1);
                let arg: Expr = args[0].clone();
                let typed_arg = arg.typeinfer(ctx.clone())?;

                let typ: Arc<Type> = Type::Word(*n).into();
                Ok(ExprNode::MethodCall(ExprMethodCall(typed_subject, method.into(), vec![typed_arg])).with_type(typ))
            },
            (Type::Word(n), method@"get") => {
                if !is_pow2(*n) {
                    return Err(TypeError::Other(format!("{method} can only be used on Word[n] where n is a power of 2")));
                }
                assert_eq!(args.len(), 1);
                let i = log2(*n);
                let arg: Expr = args[0].clone();
                let typed_arg = arg.typecheck(ctx.clone(), Type::Word(i).into())?;

                let typ: Arc<Type> = Type::Word(1).into();
                Ok(ExprNode::MethodCall(ExprMethodCall(typed_subject, method.into(), vec![typed_arg])).with_type(typ))
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
            (Type::Word(1), "mux") => {
                let c = subject_value.unwrap_word() == 1;
                if c {
                    arg_values[0].clone()
                } else {
                    arg_values[1].clone()
                }
            },
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
            (Type::Word(n), "sll") => {
                let v = subject_value.unwrap_word() << arg_values.first().unwrap().unwrap_word();
                Value::Word(*n, v % (1 << n))
            },
            (Type::Word(n), "srl") => {
                let v = subject_value.unwrap_word() >> arg_values.first().unwrap().unwrap_word();
                Value::Word(*n, v % (1 << n))
            },
            (Type::Word(_n), "get") => {
                let v = subject_value.unwrap_word();
                let idx = arg_values.first().unwrap().unwrap_word();

                // eg, 1 in position self.index(), or else all zeroes
                let v_index_masked = v & (1 << idx);
                // v_new should be 0b000.....01
                // eg, 1 in index 0, or else all zeroes
                let v_new = v_index_masked >> idx;
                assert!(v_new == 0 || v_new == 1);
                Value::Word(1, v_new)
            },
            _ => panic!(),
        };
        assert_eq!(result_value.type_of(), typ);
        result_value
    }
}

fn is_pow2(n: u64) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

fn log2(n: u64) -> u64 {
    assert!(is_pow2(n));
    63 - (n.leading_zeros() as u64)
}
