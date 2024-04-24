pub mod ast;
pub mod context;
pub mod expr;
pub mod value;

lalrpop_mod!(grammar);

use lalrpop_util::lalrpop_mod;
use context::Context;
use ast::*;
use value::*;
use expr::*;

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
                    let type_ctx = value_context_to_type_context(ctx.clone());
                    let typed_expr = typeinfer(type_ctx, &*expr);
                    println!("{}", eval(ctx, &typed_expr));
                },
                Err(err) => eprintln!("{err:?}"),
            }
        }
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
