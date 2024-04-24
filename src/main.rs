mod ast;
mod context;

lalrpop_mod!(grammar);

use lalrpop_util::lalrpop_mod;
use lalrpop_util::ParseError;
use context::Context;
use ast::*;

fn main() {
    loop {
        let mut input = String::new();
        print!(">>> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        if let Ok(_) = std::io::stdin().read_line(&mut input) {
            match grammar::ExprParser::new().parse(&input) {
                Ok(expr) => {
                    let ctx = Context::from(vec![
                        ("x".into(), Value::Word(8, 1)),
                        ("y".into(), Value::Word(8, 2)),
                        ("z".into(), Value::Word(8, 4)),
                        ("zero".into(), Value::Word(8, 0)),
                        ("buffer.out".into(), Value::Word(8, 42)),
                    ]);
                    println!("{}", eval(ctx, &expr));
                },
                Err(ParseError::UnrecognizedEof {..}) => break,
                Err(err) => eprintln!("{err:?}"),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    X,
    Bool(bool),
    Word(Width, u64),
    Vec(Vec<Value>),
    Struct(Type, Vec<(Field, Value)>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::X => write!(f, "XXX"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Word(w, n) => write!(f, "{n}w{w}"),
            Value::Vec(vs) => {
                write!(f, "[")?;
                for (i, v) in vs.iter().enumerate() {
                    if i + 1 < vs.len() {
                        write!(f, "{v}, ")?;
                    } else {
                        write!(f, "{v}")?;
                    }
                }
                write!(f, "]")
            },
            Value::Struct(typ, fields) => {
                write!(f, "struct {typ} {{ ")?;
                for (i, (fld, v)) in fields.iter().enumerate() {
                    write!(f, "{fld} = {v}")?;
                    if i + 1 < fields.len() {
                        write!(f, ", ")?;
                    } else {
                        write!(f, " ")?;
                    }
                }
                write!(f, "}}")
            },
//            Value::Enum(typedef, name) => write!(f, "{}::{}", typedef.name(), name),
//            Value::Ctor(ctor, vs) => {
//                write!(f, "@{ctor}")?;
//                if vs.len() > 0 {
//                    write!(f, "(")?;
//                    for (i, v) in vs.iter().enumerate() {
//                        write!(f, "{v:?}")?;
//                        if i + 1 < vs.len() {
//                            write!(f, ", ")?;
//                        }
//                    }
//                    write!(f, ")")
//                } else {
//                    Ok(())
//                }
//            },
        }
    }
}

pub fn eval(ctx: Context<Path, Value>, expr: &Expr) -> Value {
    match expr {
        Expr::Reference(r) => ctx.lookup(r).unwrap(),
        Expr::Word(w) => Value::Word(w.width().unwrap(), w.val()),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Vec(es) => {
            let vs = es.iter().map(|e| eval(ctx.clone(), e)).collect::<Vec<Value>>();
            Value::Vec(vs)
        },
        Expr::Struct(typ, fields) => {
            let vs: Vec<(Field, Value)> = fields.iter().map(|(f, fe)| (f.clone(), eval(ctx.clone(), fe))).collect::<Vec<_>>();
            Value::Struct(typ.clone(), vs)
        },
        Expr::UnOp(_op, _a0) => todo!(),
        Expr::BinOp(op, a0, a1) => {
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
        Expr::Call(_name, _es) => {
            todo!()
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
                Value::Vec(vs) => {
                    let mut rs = vs.clone();
                    for edit in edits {
                        if let WithEdit::Idx(i, e_i) = edit {
                            rs[*i as usize] = eval(ctx.clone(), e_i);
                        } else {
                            panic!("Invalid with edit")
                        }
                    }
                    Value::Vec(rs)
                },
//                Value::Struct(vs) => {
//                },
              _ => panic!("Invalid value for with expression."),
            }
        },
    }
}

#[test]
fn parse_exprs() {
    let expr_strs = vec![
        "0",
        "1_000",
        "0b1010",
        "2w8",
        "0b1010w4",
//        "0xff",
//        "0xffw16",
        "x",
        "x.y",
        "x.y.z",
        "x + y",
        "x - y",
        "x ++ y",
        "x || y",
        "x ^ y",
        "x && y",
        "x == y",
        "x != y",
        "x < y",
        "x <= y",
        "x > y",
        "x >= y",
        "[]",
        "[0, 1, 1, 2, 3, 5]",
        "(x)",
        "cat(x, y, z)",
        "f(x, y)",
        "x->real",
        "x[0]",
        "x[8..0]",
//        "x[i]",
        "struct Unit {}",
        "struct Complex { real = 0w8, imag = 1w8 }",
        "struct Complex { real = 0w8, imag = 1w8 }",
        "
            with x {
                this[0] = 1w8;
                this[2] = 7w8;
            }
        ",
    ];
    for expr_str in expr_strs {
        eprintln!("Testing {expr_str:?}");
        let _expr: Box<Expr> = grammar::ExprParser::new().parse(expr_str).unwrap();
    }
}
