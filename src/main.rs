use virdant::context::Context;
use virdant::parse::{parse_package, parse_expr};
use virdant::value::*;
use virdant::sim::*;
use virdant::hir::*;
use virdant::common::*;
use virdant::types::Type;
use virdant::checker;

fn main() {
    sim();
}

pub fn sim() {
    let package = std::fs::read_to_string("examples/sim.vir").unwrap();

    let mut sim = virdant::sim::simulator(&package, "Top").unwrap();
    println!("################################################################################");
    println!("Initial");
    println!("{sim}");

    sim.poke("top.in".into(), Value::Word(8, 10));
    println!("################################################################################");
    println!("poke top.in = 10w8");
    println!("{sim}");

//    sim.reset();
//    println!("################################################################################");
//    println!("reset");
//    println!("{sim}");

    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");

    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");

    loop {
    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");
    std::thread::sleep(std::time::Duration::from_millis(400));
    }
}


pub fn mlir() {
    let package_text = "

        public module Top {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8] := in;
            reg buf : Word[8] on clk <= in->add(1);
        }

        module Foo {
        }

    ";
    checker::compile(package_text).unwrap();
}

pub fn parse() {
    let package = parse_package("

        public module Top {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8];

            submodule buf of Buffer;
            buf.in := in;

            out := buf.out->add(in);
        }

        module Buffer {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8] := in;
            reg buf : Word[8] on clk <= in;
        }

    ").unwrap();
    dbg!(package);
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
                    let untyped_expr = Expr::from_ast(&expr);
                    match untyped_expr.typeinfer(type_ctx) {
                        Err(e) => eprintln!("ERROR: {e:?}"),
                        Ok(typed_expr) => println!("{}", typed_expr.eval(ctx)),
                    }
                },
                Err(err) => eprintln!("{err:?}"),
            }
        }
    }
}

pub fn value_context_to_type_context(ctx: Context<Path, Value>) -> Context<Path, Arc<Type>> {
    let new_ctx: Vec<(Path, Arc<Type>)> = ctx.into_inner().into_iter().map(|(path, value)| (path, value.type_of().into())).collect();
    Context::from(new_ctx)
}
