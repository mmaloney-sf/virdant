use crate::context::Context;
use crate::value::Value;
use crate::ast::*;
use crate::types::Type;
use crate::expr::*;

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
