use std::collections::HashSet;

use crate::context::Context;
use crate::value::Value;
use crate::ast::*;
use crate::types::Type;

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
