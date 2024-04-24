use crate::context::Context;
use crate::value::Value;
use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Bool,
    Word(Width),
    Vec(Box<Type>, usize),
    Other(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Type::Bool => write!(f, "Bool"),
            Type::Word(width) => write!(f, "Word<{width}>"),
            Type::Vec(typ, n) => write!(f, "Vec<{typ}, {n}>"),
            Type::Other(typename) => write!(f, "{typename}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypedWithEdit {
    Idx(u64, Box<TypedExpr>),
    Field(Field, Box<TypedExpr>),
}

#[derive(Debug, Clone)]
pub enum TypedExpr {
    Reference(Type, Path),
    Word(Width, u64),
    Bool(bool),
    Vec(Type, Vec<TypedExpr>),
    UnOp(Type, UnOp, Box<TypedExpr>),
    BinOp(Type, BinOp, Box<TypedExpr>, Box<TypedExpr>),
    Struct(Type, Vec<(Field, Box<TypedExpr>)>),
//    If(Box<TypedExpr>, Box<TypedExpr>, Box<TypedExpr>),
//    Match(Box<TypedExpr>, Vec<MatchArm>),
//    Let(Ident, Option<Type>, Box<TypedExpr>, Box<TypedExpr>),
    FnCall(Ident, Vec<TypedExpr>),
    MethodCall(Box<TypedExpr>, Ident, Vec<TypedExpr>),
    Cat(Vec<TypedExpr>),
    IdxField(Box<TypedExpr>, Ident),
    Idx(Box<TypedExpr>, u64),
    IdxRange(Box<TypedExpr>, u64, u64),
    With(Box<TypedExpr>, Vec<TypedWithEdit>),
}

impl TypedExpr {
    pub fn type_of(&self) -> Type {
        match self {
            TypedExpr::Reference(typ, _path) => typ.clone(),
            TypedExpr::Word(width, _value) => Type::Word(*width),
            TypedExpr::Bool(_b) => Type::Bool,
            TypedExpr::Vec(typ, es) => Type::Vec(Box::new(typ.clone()), es.len()),
            TypedExpr::UnOp(typ, _op, _e0) => typ.clone(),
            TypedExpr::BinOp(typ, _op, _e0, _e1) => typ.clone(),
            TypedExpr::Struct(typ, _flds) => Type::Other(typ.to_string()),
        //    TypedExpr::If(Box<Expr>, Box<Expr>, Box<Expr>) => todo!(),
        //    TypedExpr::Match(Box<Expr>, Vec<MatchArm>) => todo!(),
        //    TypedExpr::Let(Ident, Option<Type>, Box<Expr>, Box<Expr>) => todo!(),
//            TypedExpr::Call(Ident, Vec<Expr>) => todo!(),
//            TypedExpr::Cat(Vec<Expr>) => todo!(),
//            TypedExpr::IdxField(Box<Expr>, Ident) => todo!(),
//            TypedExpr::Idx(Box<Expr>, u64) => todo!(),
//            TypedExpr::IdxRange(Box<Expr>, u64, u64) => todo!(),
//            TypedExpr::With(Box<Expr>, Vec<WithEdit>) => todo!(),
            _ => todo!()
        }
    }
}
pub fn eval(ctx: Context<Path, Value>, expr: &TypedExpr) -> Value {
    match expr {
        TypedExpr::Reference(typ, r) => ctx.lookup(r).unwrap(),
        TypedExpr::Word(width, value) => Value::Word(*width, *value),
        TypedExpr::Bool(b) => Value::Bool(*b),
        TypedExpr::Vec(typ, es) => {
            let vs = es.iter().map(|e| eval(ctx.clone(), e)).collect::<Vec<Value>>();
            Value::Vec(typ.clone(), vs)
        },
        TypedExpr::Struct(typ, fields) => {
            let vs: Vec<(Field, Value)> = fields.iter().map(|(f, fe)| (f.clone(), eval(ctx.clone(), fe))).collect::<Vec<_>>();
            Value::Struct(typ.clone(), vs)
        },
        TypedExpr::UnOp(typ, _op, _a0) => todo!(),
        TypedExpr::BinOp(typ, op, a0, a1) => {
            let v0 = eval(ctx.clone(), a0);
            let v1 = eval(ctx.clone(), a1);
            match op.as_str() {
                "&&" | "||" | "^" => {
                    if let (Value::Bool(b0), Value::Bool(b1)) = (v0, v1) {
                        match op.as_str() {
                            "&&" => Value::Bool(b0 && b1),
                            "||" => Value::Bool(b0 || b1),
                            "^" => Value::Bool(b0 ^ b1),
                            _ => panic!(),
                        }
                    } else {
                         panic!()
                    }
                },
                "+" | "++" | "-" => {
                    if let (Value::Word(w0, x0), Value::Word(w1, x1)) = (v0, v1) {
                        match op.as_str() {
                            "+" => Value::Word(w0.max(w1), x0 + x1),
                            "++" => Value::Word(w0.max(w1) + 1, x0 + x1),
                            "-" => Value::Word(w0.max(w1), x0 - x1),
                            _ => panic!(),
                        }
                    } else {
                        panic!("Numeric binop had a problem");
                    }
                },
                _ => panic!("Unknown binary op {op:?}"),
            }
        },
        TypedExpr::FnCall(_name, _es) => {
            todo!()
        },
        TypedExpr::MethodCall(_s, _name, _es) => {
            todo!()
        },
        TypedExpr::Cat(_es) => {
//            let vs = es.iter().map(|e| eval(ctx.clone(), e)).collect::<Vec<Value>>();
//            let mut w = 0;
//            Value::Word(vs)
            todo!()
        }
        TypedExpr::IdxField(s, f) => {
            /*
            let v = eval(ctx.clone(), s);
            if let Value::Struct(_structname, flds) = v {
                for (fname, fe) in &flds {

                }
                panic!()
            } else {
                panic!()
            }
            */
            todo!()
        },
        TypedExpr::Idx(_s, _i) => todo!(),
        TypedExpr::IdxRange(_s,  _i,  _j) => todo!(),
        TypedExpr::With(s,  edits) => {
            let v = eval(ctx.clone(), s);
            match v {
                Value::Vec(typ, vs) => {
                    let mut rs = vs.clone();
                    for edit in edits {
                        if let TypedWithEdit::Idx(i, e_i) = edit {
                            rs[*i as usize] = eval(ctx.clone(), e_i);
                        } else {
                            panic!("Invalid with edit")
                        }
                    }
                    Value::Vec(typ, rs)
                },
//                Value::Struct(vs) => {
//                },
              _ => panic!("Invalid value for with expression."),
            }
        },
    }
}

pub fn value_context_to_type_context(ctx: Context<Path, Value>) -> Context<Path, Type> {
    let new_ctx: Vec<(Path, Type)> = ctx.into_inner().into_iter().map(|(path, value)| (path, value.type_of())).collect();
    Context::from(new_ctx)
}

pub fn typeinfer(ctx: Context<Path, Type>, expr: &Expr) -> TypedExpr {
    match expr {
        Expr::Reference(path) => {
            let typ = ctx.lookup(path).unwrap();
            TypedExpr::Reference(typ, path.clone())
        },
        Expr::Word(WordLit(width, value)) => TypedExpr::Word(width.unwrap(), *value),
        Expr::Bool(b) => TypedExpr::Bool(*b),
        Expr::Vec(es) => {
            let typed_es: Vec<TypedExpr> = es.iter().map(|e| typeinfer(ctx.clone(), e)).collect();
            let typ = typed_es.first().unwrap().type_of();
            TypedExpr::Vec(typ, typed_es)
        },
//        Expr::UnOp(typ, UnOp, Box<TypedExpr>),
//        Expr::BinOp(typ, BinOp, Box<TypedExpr>, Box<TypedExpr>),
//        Expr::Struct(typ, Vec<(Field, Box<TypedExpr>)>),
    //    If(Box<Expr>, Box<TypedExpr>, Box<TypedExpr>),
    //    Match(Box<Expr>, Vec<MatchArm>),
    //    Let(Ident, Option<Type>, Box<Expr>, Box<TypedExpr>),
//        Expr::Call(Ident, Vec<TypedExpr>),
//        Expr::Cat(Vec<TypedExpr>),
//        Expr::IdxField(Box<TypedExpr>, Ident),
//        Expr::Idx(Box<TypedExpr>, u64),
//        Expr::IdxRange(Box<TypedExpr>, u64, u64),
//        Expr::With(Box<TypedExpr>, Vec<TypedWithEdit>),
        _ => todo!()
    }
}
