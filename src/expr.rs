use std::collections::HashSet;

use crate::context::Context;
use crate::value::Value;
use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unknown,
    Clock,
    Bool,
    Word(Width),
    Vec(Box<Type>, usize),
    Other(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Type::Unknown => write!(f, "UNKNOWN"),
            Type::Clock => write!(f, "Clock"),
            Type::Bool => write!(f, "Bool"),
            Type::Word(width) => write!(f, "Word<{width}>"),
            Type::Vec(typ, n) => write!(f, "Vec<{typ}, {n}>"),
            Type::Other(typename) => write!(f, "{typename}"),
        }
    }
}

impl Expr {
    pub fn type_of(&self) -> Type {
        match self {
            Expr::Reference(typ, _path) => typ.clone(),
            Expr::Word(width, _value) => Type::Word(width.unwrap()),
            Expr::Bool(_b) => Type::Bool,
            Expr::Vec(typ, es) => Type::Vec(Box::new(typ.clone()), es.len()),
            Expr::Struct(typ, _flds) => Type::Other(typ.to_string()),
        //    Expr::If(Box<Expr>, Box<Expr>, Box<Expr>) => todo!(),
        //    Expr::Match(Box<Expr>, Vec<MatchArm>) => todo!(),
        //    Expr::Let(Ident, Option<Type>, Box<Expr>, Box<Expr>) => todo!(),
//            Expr::Call(Ident, Vec<Expr>) => todo!(),
//            Expr::Cat(Vec<Expr>) => todo!(),
//            Expr::IdxField(Box<Expr>, Ident) => todo!(),
//            Expr::Idx(Box<Expr>, u64) => todo!(),
//            Expr::IdxRange(Box<Expr>, u64, u64) => todo!(),
//            Expr::With(Box<Expr>, Vec<WithEdit>) => todo!(),
            _ => todo!()
        }
    }

    pub fn references(&self) -> HashSet<Path> {
        match self {
            Expr::Reference(_typ, path) => vec![path.clone()].into_iter().collect(),
            Expr::Word(_width, _value) => HashSet::new(),
            Expr::Bool(_b) => HashSet::new(),
            Expr::Vec(_typ, es) => {
                let mut result = HashSet::new();
                for e in es {
                    result.extend(e.references());
                }
                result
            },
            Expr::MethodCall(_typ, subject, _fun, args) => {
                let mut result = HashSet::new();
                result.extend(subject.references());
                for e in args {
                    result.extend(e.references());
                }
                result
            },
            //Expr::Struct(typ, _flds) => Type::Other(typ.to_string()),
        //    Expr::If(Box<Expr>, Box<Expr>, Box<Expr>) => todo!(),
        //    Expr::Match(Box<Expr>, Vec<MatchArm>) => todo!(),
        //    Expr::Let(Ident, Option<Type>, Box<Expr>, Box<Expr>) => todo!(),
//            Expr::Cat(Vec<Expr>) => todo!(),
//            Expr::IdxField(Box<Expr>, Ident) => todo!(),
//            Expr::Idx(Box<Expr>, u64) => todo!(),
//            Expr::IdxRange(Box<Expr>, u64, u64) => todo!(),
//            Expr::With(Box<Expr>, Vec<WithEdit>) => todo!(),
            _ => todo!()
        }
    }
}

pub fn eval(ctx: Context<Path, Value>, expr: &Expr) -> Value {
    match expr {
        Expr::Reference(typ, r) => ctx.lookup(r).unwrap(),
        Expr::Word(width, value) => Value::Word(width.unwrap(), *value),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Vec(typ, es) => {
            let vs = es.iter().map(|e| eval(ctx.clone(), e)).collect::<Vec<Value>>();
            Value::Vec(typ.clone(), vs)
        },
        Expr::Struct(typ, fields) => {
            let vs: Vec<(Field, Value)> = fields.iter().map(|(f, fe)| (f.clone(), eval(ctx.clone(), fe))).collect::<Vec<_>>();
            Value::Struct(typ.clone(), vs)
        },
        Expr::FnCall(_name, _es) => {
            todo!()
        },
        // a->foo(x)
        Expr::MethodCall(_typ, subject, name, args) => {
            let subject_value: Value = eval(ctx.clone(), subject);
            let arg_values: Vec<Value> = args.iter().map(|arg| eval(ctx.clone(), arg)).collect();

            match (subject.type_of(), name.as_str()) {
                (Type::Word(_n), "eq") => {
                    Value::Bool(subject_value == *arg_values.first().unwrap())
                },
                (Type::Word(n), "add") => {
                    let a = subject_value.unwrap_word();
                    let b = arg_values.first().unwrap().unwrap_word();
                    Value::Word(n, a.wrapping_add(b) % (1 << n))
                },
                _ => panic!(),
            }
        },
        Expr::Cat(_es) => {
//            let vs = es.iter().map(|e| eval(ctx.clone(), e)).collect::<Vec<Value>>();
//            let mut w = 0;
//            Value::Word(vs)
            todo!()
        }
        Expr::IdxField(s, f) => {
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
        Expr::Idx(_s, _i) => todo!(),
        Expr::IdxRange(_s,  _i,  _j) => todo!(),
        Expr::With(s,  edits) => {
            let v = eval(ctx.clone(), s);
            match v {
                Value::Vec(typ, vs) => {
                    let mut rs = vs.clone();
                    for edit in edits {
                        if let WithEdit::Idx(i, e_i) = edit {
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

pub fn typecheck(ctx: Context<Path, Type>, expr: &Expr, typ: Type) -> Expr {
    let result = match expr {
        Expr::Reference(_typ, path) => {
            let type_actual = ctx.lookup(path).unwrap();
            assert_eq!(type_actual, typ);
            Expr::Reference(type_actual, path.clone())
        },
        Expr::Word(width, value) => {
            if let Type::Word(n) = typ {
                if let Some(width) = width {
                    assert_eq!(*width, n);
                    typeinfer(ctx.clone(), expr)
                } else {
                    if fits_in(*value, n) {
                        Expr::Word(Some(n), *value)
                    } else {
                        panic!()
                    }
                }
            } else {
                panic!()
            }
        }
        Expr::Bool(_b) => typeinfer(ctx.clone(), expr),
        Expr::Vec(_typ, es) => {
            let typed_es: Vec<Expr> = es.iter().map(|e| typeinfer(ctx.clone(), e)).collect();
            let typ = typed_es.first().unwrap().type_of();
            Expr::Vec(typ, typed_es)
        },
//        Expr::Struct(typ, Vec<(Field, Box<Expr>)>),
    //    If(Box<Expr>, Box<Expr>, Box<Expr>),
    //    Match(Box<Expr>, Vec<MatchArm>),
    //    Let(Ident, Option<Type>, Box<Expr>, Box<Expr>),
//        Expr::FnCall(Ident, Vec<Expr>),
//        a->foo(b)
        Expr::MethodCall(_typ, _subject, _method, _args) => typeinfer(ctx.clone(), expr),
//        Expr::Cat(Vec<Expr>),
//        Expr::IdxField(Box<Expr>, Ident),
//        Expr::Idx(Box<Expr>, u64),
//        Expr::IdxRange(Box<Expr>, u64, u64),
//        Expr::With(Box<Expr>, Vec<WithEdit>),
        _ => {
            dbg!(expr, typ);
            todo!()
        }
    };
    assert_eq!(result.type_of(), typ.clone());
    result
}

pub fn typeinfer(ctx: Context<Path, Type>, expr: &Expr) -> Expr {
    match expr {
        Expr::Reference(_typ, path) => {
            let typ = ctx.lookup(path).unwrap();
            Expr::Reference(typ, path.clone())
        },
        Expr::Word(width, value) => Expr::Word(Some(width.unwrap()), *value),
        Expr::Bool(b) => Expr::Bool(*b),
        Expr::Vec(_typ, es) => {
            let typed_es: Vec<Expr> = es.iter().map(|e| typeinfer(ctx.clone(), e)).collect();
            let typ = typed_es.first().unwrap().type_of();
            Expr::Vec(typ, typed_es)
        },
//        Expr::Struct(typ, Vec<(Field, Box<Expr>)>),
    //    If(Box<Expr>, Box<Expr>, Box<Expr>),
    //    Match(Box<Expr>, Vec<MatchArm>),
    //    Let(Ident, Option<Type>, Box<Expr>, Box<Expr>),
//        Expr::FnCall(Ident, Vec<Expr>),
//        a->foo(b)
        Expr::MethodCall(_typ, subject, method, args) => {
            let typed_subject: Expr = typeinfer(ctx.clone(), subject);
            let subject_type: Type = typed_subject.type_of();
            match (subject_type, method.as_str()) {
                (Type::Word(n), "eq") => {
                    assert_eq!(args.len(), 1);
                    let typed_arg = typecheck(ctx.clone(), &args.first().unwrap(),  Type::Word(n));
                    Expr::MethodCall(Type::Bool, Box::new(typed_subject), "eq".to_string(), vec![typed_arg])
                },
                // 1w8->add(2)
                (Type::Word(n), "add") => {
                    assert_eq!(args.len(), 1);
                    let typed_arg = typecheck(ctx.clone(), &args.first().unwrap(),  Type::Word(n));
                    Expr::MethodCall(Type::Word(n), Box::new(typed_subject), "add".to_string(), vec![typed_arg])
                },
                _ => panic!(),
            }
        },
//        Expr::Cat(Vec<Expr>),
//        Expr::IdxField(Box<Expr>, Ident),
//        Expr::Idx(Box<Expr>, u64),
//        Expr::IdxRange(Box<Expr>, u64, u64),
//        Expr::With(Box<Expr>, Vec<WithEdit>),
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
