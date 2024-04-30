use crate::common::*;
use crate::value::Value;
use crate::context::Context;
use crate::types::Type;
use crate::parse::{parse_package, parse_expr};
use crate::hir::Package;
use crate::ast;
use crate::hir::*;

#[test]
fn test_examples() {
    let examples_dir = std::path::Path::new("examples");
    let mut errors = vec![];

    if let Ok(entries) = std::fs::read_dir(examples_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let filename = entry.file_name();
                if let Some(filename) = filename.to_str() {
                    if filename.ends_with(".vir") {
                        let text = match std::fs::read_to_string(entry.path()) {
                            Ok(text) => text,
                            Err(_) => panic!("Failed to read file {:?}", entry.path()),
                        };

                        if let Err(_error) = std::panic::catch_unwind(|| {
                            let package = Package::from_ast(&parse_package(&text).expect(&format!("Testing {:?}", entry.path())));
                            package.check().expect(&format!("Failed to check: {filename}"));
                        }) {
                            errors.push(filename.to_string());
                        }
                    }
                }
            }
        }
    } else {
        panic!("Failed to read examples directory");
    }

    if errors.len() > 0 {
        panic!("Errors in examples:\n  - {}", errors.join("\n  - "))
    }
}

#[test]
fn test_parse_exprs() {
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
        "[]",
        "[0, 1, 1, 2, 3, 5]",
        "(x)",
//        "cat(x, y, z)",
//        "f(x, y)",
        "z->f(x, y)",
        "a->eq(b)",
        "a->lt(b)",
        "a->lte(b)",
//        "x->real",
//        "x[0]",
//        "x[8..0]",
//        "x[i]",
//        "struct Unit {}",
//        "struct Complex { real = 0w8, imag = 1w8 }",
//        "struct Complex { real = 0w8, imag = 1w8 }",
        /*
        "
            with x {
                this[0] = 1w8;
                this[2] = 7w8;
            }
        ",
        */
    ];
    for expr_str in expr_strs {
        eprintln!("Testing {expr_str:?}");
        let _expr: ast::Expr = parse_expr(expr_str).unwrap();
    }
}

#[test]
fn test_parse_package() {
    let package_text = std::fs::read_to_string("examples/hello.vir").unwrap();
    let package = parse_package(&package_text).unwrap();
    dbg!(package);
}

#[test]
fn path_tests() {
    let p1: Path = "top.foo".into();
    let p2: Path = "top.foo.bar".into();
    assert_eq!(p2.parent(), p1);
}

#[test]
fn test_typeinfer_exprs() {
    use crate::ast;
    use crate::parse;

    let expr_strs = vec![
//        "0",
//        "1_000",
//        "0b1010",
        "2w8",
        "0b1010w4",
//        "0xff",
//        "0xffw16",
        "x",
        "m.y",
//        "x.y.z",
//        "[]",
//        "[0, 1, 1, 2, 3, 5]",
        "(x)",
//        "cat(x, y, z)",
//        "f(x, y)",
//        "z->f(x, y)",
//        "a->eq(b)",
//        "a->lt(b)",
//        "a->lte(b)",
//        "x->real",
//        "x[0]",
//        "x[8..0]",
//        "x[i]",
//        "struct Unit {}",
//        "struct Complex { real = 0w8, imag = 1w8 }",
//        "struct Complex { real = 0w8, imag = 1w8 }",
        /*
        "
            with x {
                this[0] = 1w8;
                this[2] = 7w8;
            }
        ",
        */
    ];
    for expr_str in expr_strs {
        eprintln!("Testing {expr_str:?}");
        let expr: ast::Expr = parse::parse_expr(expr_str).unwrap();
        let type_ctx = Context::from(vec![
              ("x".into(), Type::Word(8).into()),
              ("m.y".into(), Type::Word(8).into()),
        ]);
        let expr = Expr::from_ast(&expr).typeinfer(type_ctx).unwrap();
        let ctx = Context::from(vec![
              ("x".into(), Value::Word(8, 1)),
              ("m.y".into(), Value::Word(8, 1)),
        ]);
        expr.eval(ctx);
    }
}

#[test]
fn test_typecheck_exprs() {
    use crate::ast;
    use crate::parse;

    let expr_strs = vec![
        ("0", Type::Word(1)),
        ("1_000", Type::Word(10)),
        ("0b1010", Type::Word(4)),
        ("2w8", Type::Word(8)),
        ("0b1010w4", Type::Word(4)),
//        ("0xff", Type::Word(8)),
//        ("0xffw16", Type::Word(8)),
        ("x", Type::Word(8)),
        ("m.y", Type::Word(8)),
//        ("x.y.z", Type::Word(8)),
//        ("[]", Type::Word(8)),
//        ("[0, 1, 1, 2, 3, 5]", Type::Word(8)),
        ("(x)", Type::Word(8)),
//        ("cat(x, y, z)", Type::Word(8)),
//        ("f(x, y)", Type::Word(8)),
//        ("z->f(x, y)", Type::Word(8)),
//        ("a->eq(b)", Type::Word(8)),
//        ("a->lt(b)", Type::Word(8)),
//        ("a->lte(b)", Type::Word(8)),
//        ("x->real", Type::Word(8)),
//        ("x[0]", Type::Word(8)),
//        ("x[8..0]", Type::Word(8)),
//        ("x[i]", Type::Word(8)),
//        ("struct Unit {}", Type::Word(8)),
//        ("struct Complex { real = 0w8, imag = 1w8 }", Type::Word(8)),
//        ("struct Complex { real = 0w8, imag = 1w8 }", Type::Word(8)),
        /*
        "
            with x {
                this[0] = 1w8;
                this[2] = 7w8;
            }
        ",
        */
    ];
    for (expr_str, typ) in expr_strs {
        eprintln!("Testing {expr_str:?}");
        let expr: ast::Expr = parse::parse_expr(expr_str).unwrap();
        let type_ctx = Context::from(vec![
              ("x".into(), Type::Word(8).into()),
              ("m.y".into(), Type::Word(8).into()),
        ]);
        let expr = Expr::from_ast(&expr).typecheck(type_ctx, typ.into()).unwrap();
        let ctx = Context::from(vec![
              ("x".into(), Value::Word(8, 1)),
              ("m.y".into(), Value::Word(8, 1)),
        ]);
        expr.eval(ctx);
    }
}
