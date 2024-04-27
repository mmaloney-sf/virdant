use crate::*;
use crate::common::*;

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
                    Expr::MethodCall(Type::Bool, Box::new(typed_subject), "eq".into(), vec![typed_arg])
                },
                // 1w8->add(2)
                (Type::Word(n), "add") => {
                    assert_eq!(args.len(), 1);
                    let typed_arg = typecheck(ctx.clone(), &args.first().unwrap(),  Type::Word(n));
                    Expr::MethodCall(Type::Word(n), Box::new(typed_subject), "add".into(), vec![typed_arg])
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
