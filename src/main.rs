use lalrpop_util::lalrpop_mod;

mod context;
lalrpop_mod!(grammar);

use context::Context;

pub type Ident = String;
pub type Width = usize;
pub type UnOp = String;
pub type BinOp = String;
pub type Type = String;
pub type Field = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    Local(Ident),
    Nonlocal(Ident, Ident),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WordLit(pub Option<Width>, pub u64);

impl WordLit {
    pub fn width(&self) -> Option<Width> {
        self.0
    }

    pub fn val(&self) -> u64 {
        self.1
    }
}

#[derive(Debug, Clone)]
pub enum WithEdit {
    Idx(u64, Box<Expr>),
    Field(Field, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Reference(Reference),
    Word(WordLit),
    Vec(Vec<Expr>),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    Struct(Type, Vec<(Field, Box<Expr>)>),
//    If(Box<Expr>, Box<Expr>, Box<Expr>),
//    Match(Box<Expr>, Vec<MatchArm>),
//    Let(Ident, Option<Type>, Box<Expr>, Box<Expr>),
    Call(Ident, Vec<Expr>),
    Cat(Vec<Expr>),
    IdxField(Box<Expr>, Ident),
    Idx(Box<Expr>, u64),
    IdxRange(Box<Expr>, u64, u64),
    With(Box<Expr>, Vec<WithEdit>),
}

fn main() {
    println!("Hello, world!");
//    let expr_text = "with [3w8, 4w8, 3w8 + 4w8] {
//        this[0] = zero;
//        this[1] = buffer.out;
//    }";
    let expr_text = "struct Complex { real = 0w8, imag = 1w8 }";

    let expr: Box<Expr> = grammar::ExprParser::new().parse(&expr_text).unwrap();
    let ctx = Context::from(vec![
        (Reference::Local("zero".to_string()), Value::Word(8, 0)),
        (Reference::Nonlocal("buffer".to_string(), "out".to_string()), Value::Word(8, 42)),
    ]);
    println!("{}", eval(ctx, &expr));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    X,
    Word(Width, u64),
    Vec(Vec<Value>),
    Struct(Type, Vec<(Field, Value)>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::X => write!(f, "XXX"),
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

pub fn eval(ctx: Context<Reference, Value>, expr: &Expr) -> Value {
    match expr {
        Expr::Reference(r) => ctx.lookup(r).unwrap(),
        Expr::Word(w) => Value::Word(w.width().unwrap(), w.val()),
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
        Expr::IdxField(_s, _f) => todo!(),
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
