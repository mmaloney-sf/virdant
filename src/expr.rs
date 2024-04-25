use std::collections::HashSet;

use crate::context::Context;
use crate::value::Value;
use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Clock,
    Bool,
    Word(Width),
    Vec(Box<Type>, usize),
    Other(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Type::Clock => write!(f, "Clock"),
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
    Struct(Type, Vec<(Field, Box<TypedExpr>)>),
//    If(Box<TypedExpr>, Box<TypedExpr>, Box<TypedExpr>),
//    Match(Box<TypedExpr>, Vec<MatchArm>),
//    Let(Ident, Option<Type>, Box<TypedExpr>, Box<TypedExpr>),
    FnCall(Ident, Vec<TypedExpr>),
    MethodCall(Type, Box<TypedExpr>, Ident, Vec<TypedExpr>),
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

    pub fn free_refs(&self) -> HashSet<Path> {
        match self {
            TypedExpr::Reference(_typ, path) => vec![path.clone()].into_iter().collect(),
            TypedExpr::Word(_width, _value) => HashSet::new(),
            TypedExpr::Bool(_b) => HashSet::new(),
            TypedExpr::Vec(_typ, es) => {
                let mut result = HashSet::new();
                for e in es {
                    result.extend(e.free_refs());
                }
                result
            },
            TypedExpr::MethodCall(_typ, subject, _fun, args) => {
                let mut result = HashSet::new();
                result.extend(subject.free_refs());
                for e in args {
                    result.extend(e.free_refs());
                }
                result
            },
            //TypedExpr::Struct(typ, _flds) => Type::Other(typ.to_string()),
        //    TypedExpr::If(Box<Expr>, Box<Expr>, Box<Expr>) => todo!(),
        //    TypedExpr::Match(Box<Expr>, Vec<MatchArm>) => todo!(),
        //    TypedExpr::Let(Ident, Option<Type>, Box<Expr>, Box<Expr>) => todo!(),
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
        TypedExpr::FnCall(_name, _es) => {
            todo!()
        },
        TypedExpr::MethodCall(_typ, subject, name, args) => {
            let subject_value: Value = eval(ctx.clone(), subject);
            let arg_values: Vec<Value> = args.iter().map(|arg| eval(ctx.clone(), arg)).collect();

            match (subject.type_of(), name.as_str()) {
                (Type::Word(_n), "eq") => {
                    Value::Bool(subject_value == *arg_values.first().unwrap())
                },
                (Type::Word(n), "add") => {
                    let a = subject_value.unwrap_word();
                    let b = arg_values.first().unwrap().unwrap_word();
                    Value::Word(n, a.wrapping_add(b) % n)
                },
                _ => panic!(),
            }
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

pub fn typecheck(ctx: Context<Path, Type>, expr: &Expr, typ: Type) -> TypedExpr {
    let result = match expr {
        Expr::Reference(path) => {
            let type_actual = ctx.lookup(path).unwrap();
            assert_eq!(type_actual, typ);
            TypedExpr::Reference(type_actual, path.clone())
        },
        Expr::Word(WordLit(width, value)) => {
            if let Type::Word(n) = typ {
                if let Some(width) = width {
                    assert_eq!(*width, n);
                    typeinfer(ctx.clone(), expr)
                } else {
                    if fits_in(*value, n) {
                        TypedExpr::Word(n, *value)
                    } else {
                        panic!()
                    }
                }
            } else {
                panic!()
            }
        }
        Expr::Bool(_b) => typeinfer(ctx.clone(), expr),
        Expr::Vec(es) => {
            let typed_es: Vec<TypedExpr> = es.iter().map(|e| typeinfer(ctx.clone(), e)).collect();
            let typ = typed_es.first().unwrap().type_of();
            TypedExpr::Vec(typ, typed_es)
        },
//        Expr::Struct(typ, Vec<(Field, Box<TypedExpr>)>),
    //    If(Box<Expr>, Box<TypedExpr>, Box<TypedExpr>),
    //    Match(Box<Expr>, Vec<MatchArm>),
    //    Let(Ident, Option<Type>, Box<Expr>, Box<TypedExpr>),
//        Expr::FnCall(Ident, Vec<TypedExpr>),
        Expr::MethodCall(_subject, _method, _args) => typeinfer(ctx.clone(), expr),
//        Expr::Cat(Vec<TypedExpr>),
//        Expr::IdxField(Box<TypedExpr>, Ident),
//        Expr::Idx(Box<TypedExpr>, u64),
//        Expr::IdxRange(Box<TypedExpr>, u64, u64),
//        Expr::With(Box<TypedExpr>, Vec<TypedWithEdit>),
        _ => {
            dbg!(expr, typ);
            todo!()
        }
    };
    assert_eq!(result.type_of(), typ.clone());
    result
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
//        Expr::Struct(typ, Vec<(Field, Box<TypedExpr>)>),
    //    If(Box<Expr>, Box<TypedExpr>, Box<TypedExpr>),
    //    Match(Box<Expr>, Vec<MatchArm>),
    //    Let(Ident, Option<Type>, Box<Expr>, Box<TypedExpr>),
//        Expr::FnCall(Ident, Vec<TypedExpr>),
        Expr::MethodCall(subject, method, args) => {
            let typed_subject: TypedExpr = typeinfer(ctx.clone(), subject);
            let subject_type: Type = typed_subject.type_of();
            match (subject_type, method.as_str()) {
                (Type::Word(n), "eq") => {
                    assert_eq!(args.len(), 1);
                    let typed_arg = typecheck(ctx.clone(), &args.first().unwrap(),  Type::Word(n));
                    TypedExpr::MethodCall(Type::Bool, Box::new(typed_subject), "eq".to_string(), vec![typed_arg])
                },
                (Type::Word(n), "add") => {
                    assert_eq!(args.len(), 1);
                    let typed_arg = typecheck(ctx.clone(), &args.first().unwrap(),  Type::Word(n));
                    TypedExpr::MethodCall(Type::Word(n), Box::new(typed_subject), "add".to_string(), vec![typed_arg])
                },
                _ => panic!(),
            }
        },
//        Expr::Cat(Vec<TypedExpr>),
//        Expr::IdxField(Box<TypedExpr>, Ident),
//        Expr::Idx(Box<TypedExpr>, u64),
//        Expr::IdxRange(Box<TypedExpr>, u64, u64),
//        Expr::With(Box<TypedExpr>, Vec<TypedWithEdit>),
        _ => todo!()
    }
}

fn fits_in(value: u64, width: Width) -> bool {
    if width > 63 {
        false
    } else {
        value < (1 << width)
    }
}
