use crate::common::*;
use crate::parse::{parse_package, parse_expr};
use crate::ast;
use crate::db::*;

#[test]
fn test_examples() {
    let examples_dir = std::path::Path::new("examples");
    let mut errors = vec![];
    let mut db = Db::default();

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
                        db.set_source(Arc::new(text.to_string()));

                        if let Err(_error) = std::panic::catch_unwind(|| {
                            db.check().unwrap();
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
        let _expr = parse_expr(expr_str).unwrap();
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
