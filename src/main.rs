pub mod ast;
pub mod expr;
pub mod context;
pub mod eval;
pub mod value;
pub mod sim;
pub mod types;
pub mod typecheck;

use context::Context;
use ast::*;
use expr::*;
use value::*;
use eval::*;
use sim::*;
use types::*;
use typecheck::*;

fn main() {
    parse();
}

pub fn parse() {
    let package = parse_package("

        public module Top {
            input clk;
            input in;
            output out;

            Buffer buf;
            buf.in := in;

            out := buf.out->add(in);
        }

        module Buffer {
            input clk;
            input in;
            output out := in;
            reg buf on clk <= in;
        }

    ").unwrap();
    dbg!(package);
}

pub fn sim() {
    let ctx = Context::from(vec![
        ("r".into(), Type::Word(8)),
        ("in".into(), Type::Word(8)),
        ("out".into(), Type::Word(8)),
    ]);

    let out_expr = typeinfer(ctx.clone(), &parse_expr("r").unwrap());
    let r_expr = typeinfer(ctx, &parse_expr("r->add(in)").unwrap());

    let mut sim = Sim::new()
        .add_simple_node("top.out".into(), Type::Word(8), out_expr)
        .add_simple_node("top.in".into(), Type::Word(8), Expr::Word(Some(8), 1))
        .add_reg_node("top.r".into(), Type::Word(8), Some(Value::Word(8, 100)), r_expr)
        .build();

    println!("################################################################################");
    println!("Initial");
    println!("{sim}");

    sim.reset();
    println!("################################################################################");
    println!("reset");
    println!("{sim}");

    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");

    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");

    sim.poke("top.in".into(), Value::Word(8, 10));
    println!("poke top.in = 10w8");
    println!("################################################################################");
    println!("{sim}");

    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");
}

pub fn repl() {
    loop {
        let mut input = String::new();
        print!(">>> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        if let Ok(_) = std::io::stdin().read_line(&mut input) {
            match parse_expr(&input) {
                Ok(expr) => {
                    let ctx = Context::from(vec![
                        ("x".into(), Value::Word(8, 1)),
                        ("y".into(), Value::Word(8, 2)),
                        ("z".into(), Value::Word(8, 4)),
                        ("zero".into(), Value::Word(8, 0)),
                        ("buffer.out".into(), Value::Word(8, 42)),
                    ]);
                    let type_ctx = value_context_to_type_context(ctx.clone());
                    let typed_expr = typeinfer(type_ctx, &expr);
                    println!("{}", eval(ctx, &typed_expr));
                },
                Err(err) => eprintln!("{err:?}"),
            }
        }
    }
}

pub fn value_context_to_type_context(ctx: Context<Path, Value>) -> Context<Path, Type> {
    let new_ctx: Vec<(Path, Type)> = ctx.into_inner().into_iter().map(|(path, value)| (path, value.type_of())).collect();
    Context::from(new_ctx)
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
        "[]",
        "[0, 1, 1, 2, 3, 5]",
        "(x)",
        "cat(x, y, z)",
        "f(x, y)",
        "z->f(x, y)",
        "a->eq(b)",
        "a->lt(b)",
        "a->lte(b)",
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
        let _expr: Expr = parse_expr(expr_str).unwrap();
    }
}
